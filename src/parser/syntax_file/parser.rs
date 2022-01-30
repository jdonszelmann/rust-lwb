use crate::parser::syntax_file::ast::{Annotation, Constructor, Sort, SyntaxFileAst};
use crate::parser::syntax_file::character_class::CharacterClass;
use crate::parser::syntax_file::parser::ParseError::{
    DuplicateStartingRule, Expected, InvalidAnnotation, NoStartingRule, UnexpectedEndOfFile,
};
use crate::source_file::{SourceFile, SourceFileIterator};
use lazy_static::lazy_static;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("found duplicate starting rule definition found")]
    DuplicateStartingRule,

    #[error("syntax definition contains no starting rule")]
    NoStartingRule,

    #[error("unexpected character: {0}")]
    UnexpectedCharacter(char),

    #[error("invalid annotation: {0}")]
    InvalidAnnotation(String),

    #[error("unexpected character, expected: {0}")]
    Expected(String),

    #[error("end of file (though more input was expected)")]
    UnexpectedEndOfFile,

    #[error(
        "invalid character range (left side of range might have been higher than the right side)"
    )]
    InvalidCharacterRange,
}

type ParseResult<T> = Result<T, ParseError>;

lazy_static! {
    static ref SYNTAX_FILE_LAYOUT: CharacterClass =
        CharacterClass::all_in_vec(vec![' ', '\n', '\t', '\r']);
}

/// Parse a source file into a syntax file ast.
pub fn parse(f: &SourceFile) -> ParseResult<SyntaxFileAst> {
    let mut iterator = f.iter();
    parse_file(&mut iterator)
}

#[derive(Debug)]
pub enum SortOrMeta {
    Sort(Sort),
    StartRule(String),
    Layout(CharacterClass),
}

fn parse_file(i: &mut SourceFileIterator) -> ParseResult<SyntaxFileAst> {
    let mut sorts = Vec::new();
    let mut layout = CharacterClass::Nothing;
    let mut starting_rule = None;

    while let Some(_) = i.peek() {
        match parse_sort_or_meta(i)? {
            Some(SortOrMeta::Sort(c)) => sorts.push(c),
            Some(SortOrMeta::StartRule(_)) if starting_rule.is_some() => {
                return Err(DuplicateStartingRule)
            }
            Some(SortOrMeta::StartRule(c)) => starting_rule = Some(c),
            Some(SortOrMeta::Layout(c)) => layout = layout.combine(c),
            None => break,
        }

        i.skip_layout(SYNTAX_FILE_LAYOUT.clone());
        if !i.accept(';') {
            return Err(Expected("; after every rule".to_string()));
        }
    }

    Ok(SyntaxFileAst {
        sorts,
        starting_rule: starting_rule.ok_or(NoStartingRule)?,
        layout,
    })
}

fn parse_sort_or_meta(i: &mut SourceFileIterator) -> ParseResult<Option<SortOrMeta>> {
    i.skip_layout(SYNTAX_FILE_LAYOUT.clone());
    if i.exhausted() {
        return Ok(None);
    }

    if i.accept_str("layout") {
        i.skip_layout(SYNTAX_FILE_LAYOUT.clone());
        if !i.accept('=') {
            return Err(Expected("= in layout/".to_string()));
        }
        Ok(Some(SortOrMeta::Layout(parse_character_class(i)?)))
    } else if i.accept_str("start") {
        i.skip_layout(SYNTAX_FILE_LAYOUT.clone());
        if !i.accept_str("at") {
            return Err(Expected("'at' after 'start'".to_string()));
        }
        i.skip_layout(SYNTAX_FILE_LAYOUT.clone());
        Ok(Some(SortOrMeta::StartRule(parse_identifier(i)?)))
    } else {
        let name = parse_identifier(i)?;
        i.skip_layout(SYNTAX_FILE_LAYOUT.clone());

        if !i.accept('=') {
            return Err(Expected("= in rule".to_string()));
        }

        let mut constructors = vec![parse_constructor(i)?];

        while i.accept('|') {
            i.skip_layout(SYNTAX_FILE_LAYOUT.clone());
            constructors.push(parse_constructor(i)?);
        }

        i.skip_layout(SYNTAX_FILE_LAYOUT.clone());
        let annotations = parse_annotations(i)?;

        Ok(Some(SortOrMeta::Sort(Sort {
            name,
            constructors,
            annotations,
        })))
    }
}

fn parse_annotation(i: &mut SourceFileIterator) -> ParseResult<Option<Annotation>> {
    i.skip_layout(SYNTAX_FILE_LAYOUT.clone());

    if i.accept_str("no-pretty-print") {
        Ok(Some(Annotation::NoPrettyPrint))
    } else if i.accept_str("no-layout") {
        Ok(Some(Annotation::NoLayout))
    } else if i.peek() == Some(&'}') {
        Ok(None)
    } else {
        let chars: CharacterClass = SYNTAX_FILE_LAYOUT
            .clone()
            .combine(CharacterClass::all_in_vec(vec!['}', ',']));
        Err(InvalidAnnotation(i.accept_to_next(chars)))
    }
}

fn parse_annotations(i: &mut SourceFileIterator) -> ParseResult<Vec<Annotation>> {
    i.skip_layout(SYNTAX_FILE_LAYOUT.clone());

    if i.accept('{') {
        let mut annotations = vec![];

        if let Some(a) = parse_annotation(i)? {
            annotations.push(a);

            while i.accept_skip_layout(",", SYNTAX_FILE_LAYOUT.clone()) {
                if let Some(a) = parse_annotation(i)? {
                    annotations.push(a);
                }
            }
        }

        if !i.accept("}") {
            return Err(Expected("closing brace (})".to_string()));
        }

        Ok(annotations)
    } else {
        Ok(vec![])
    }
}

fn parse_number(i: &mut SourceFileIterator) -> ParseResult<u64> {
    i.skip_layout(SYNTAX_FILE_LAYOUT.clone());
    let number_char_class = CharacterClass::from('0'..='9');
    let mut res = 0;

    while let Some(i) = i.accept_option(number_char_class.clone()) {
        res *= 10;
        res += i.to_digit(10).expect("must parse") as u64;
    }

    Ok(res)
}

fn parse_constructor(i: &mut SourceFileIterator) -> ParseResult<Constructor> {
    let mut lst = vec![parse_simple_constructor(i)?];

    loop {
        let saved = i.clone();
        match parse_simple_constructor(i) {
            Ok(i) => lst.push(i),
            Err(e) => {
                *i = saved;
                break;
            }
        }
    }

    if lst.len() == 1 {
        Ok(lst.pop().unwrap())
    } else {
        Ok(Constructor::Sequence(lst))
    }
}

fn parse_simple_constructor(i: &mut SourceFileIterator) -> ParseResult<Constructor> {
    i.skip_layout(SYNTAX_FILE_LAYOUT.clone());
    let res = parse_constructor_no_suffix(i)?;
    i.skip_layout(SYNTAX_FILE_LAYOUT.clone());

    if i.accept("*") {
        Ok(Constructor::Repeat {
            c: Box::new(res),
            min: 0,
            max: None,
        })
    } else if i.accept("+") {
        Ok(Constructor::Repeat {
            c: Box::new(res),
            min: 1,
            max: None,
        })
    } else if i.accept("{") {
        let min = parse_number(i)?;
        i.skip_layout(SYNTAX_FILE_LAYOUT.clone());
        let max = if i.accept(",") {
            Some(parse_number(i)?)
        } else {
            None
        };
        i.skip_layout(SYNTAX_FILE_LAYOUT.clone());

        if !i.accept("}") {
            return Err(Expected(
                "closing brace after repetition specification".to_string(),
            ));
        }

        Ok(Constructor::Repeat {
            c: Box::new(res),
            min,
            max,
        })
    } else if i.accept("?") {
        Ok(Constructor::Repeat {
            c: Box::new(res),
            min: 0,
            max: Some(1),
        })
    } else {
        Ok(res)
    }
}

fn parse_constructor_no_suffix(i: &mut SourceFileIterator) -> ParseResult<Constructor> {
    parse_constructor_atom(i)
}

fn parse_constructor_atom(i: &mut SourceFileIterator) -> ParseResult<Constructor> {
    i.skip_layout(SYNTAX_FILE_LAYOUT.clone());

    if i.accept("(") {
        let res = parse_constructor(i)?;
        if !i.accept(")") {
            return Err(Expected("closing parenthesis".to_string()));
        } else {
            return Ok(res);
        }
    }

    if let Some(true) = i.peek().map(|c| ['\'', '"'].contains(c)) {
        return Ok(Constructor::Literal(parse_literal(i)?));
    }

    if let Some(true) = i.peek().map(|c| ['['].contains(c)) {
        return Ok(Constructor::CharacterClass(parse_character_class(i)?));
    }

    if let Ok(i) = parse_identifier(i) {
        return Ok(Constructor::Identifier(i));
    }

    Err(Expected(
        "literal, identifier or parenthesized expression".to_string(),
    ))
}

fn parse_literal(i: &mut SourceFileIterator) -> ParseResult<String> {
    i.skip_layout(SYNTAX_FILE_LAYOUT.clone());
    let mut res = String::new();
    if let Some(c) = i.accept_option("\"'") {
        let mut escaped = false;

        loop {
            let mut next_escaped = false;
            match i.next() {
                None => return Err(Expected(format!("closing quote: {c} to end string"))),
                Some('\\') if !escaped => next_escaped = true,
                Some(x) if x == c && !escaped => break,
                Some(v) => res.push(v),
            }

            escaped = next_escaped;
        }
    } else {
        return Err(Expected("literal".to_string()));
    }

    Ok(res)
}

fn parse_identifier(i: &mut SourceFileIterator) -> ParseResult<String> {
    i.skip_layout(SYNTAX_FILE_LAYOUT.clone());
    let mut res = String::new();
    if let Some(c) = i.accept_option(
        CharacterClass::from('a'..='z')
            .combine(CharacterClass::from('A'..='Z'))
            .combine("_$".into()),
    ) {
        res.push(c);

        while let Some(c) = i.accept_option(
            CharacterClass::from('a'..='z')
                .combine(CharacterClass::from('A'..='Z'))
                .combine(CharacterClass::from('0'..='9'))
                .combine("_$".into()),
        ) {
            res.push(c)
        }
    } else {
        return Err(Expected("identifier".to_string()));
    }

    Ok(res)
}

fn parse_character_class(i: &mut SourceFileIterator) -> ParseResult<CharacterClass> {
    i.skip_layout(SYNTAX_FILE_LAYOUT.clone());

    if i.accept('[') {
        let mut res = CharacterClass::Nothing;
        let mut invert = false;

        if i.accept("^") {
            invert = true;
        }

        while let Some(&c) = i.peek() {
            if c == ']' {
                break;
            }
            i.advance();
            if i.peek() == Some(&'-') {
                let lower = c;
                i.advance();
                if let Some(upper) = i.next() {
                    if lower as u32 > upper as u32 {
                        return Err(ParseError::InvalidCharacterRange);
                    }

                    res = res.combine((lower..=upper).into());

                    continue;
                }

                return Err(UnexpectedEndOfFile);
            } else {
                res = res.combine(c.into())
            }
        }

        if !i.accept(']') {
            Err(Expected("closing ] for character class".to_string()))
        } else if invert {
            Ok(res.invert())
        } else {
            Ok(res)
        }
    } else {
        Err(Expected("[ for character class".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bnf::Grammar;

    macro_rules! parse_test {
        ($name: ident test that $input: literal parses) => {
            parse_test!($name test that $input parses with parse_file)
        };

        ($name: ident test that $input: literal parses with $parse_func: ident) => {
            #[test]
            fn $name () {
                let sf = SourceFile::new_for_test($input);
                let mut sfi = sf.iter();
                let res = $parse_func(&mut sfi);
                assert!(res.is_ok(), "{:?}", res);
            }
        };
        ($name: ident test that $input: literal fails to parse with $parse_func: ident) => {
            #[test]
            fn $name () {
                let sf = SourceFile::new_for_test($input);
                let mut sfi = sf.iter();
                let res = $parse_func(&mut sfi);
                assert!(res.is_err(), "{:?}", res);
            }
        };
        ($name: ident test that $input: literal parses with $parse_func: ident to $($tt: tt)*) => {
            #[test]
            fn $name () {
                let sf = SourceFile::new_for_test($input);
                let mut sfi = sf.iter();
                let res = $parse_func(&mut sfi);
                assert!(res.is_ok(), "{:?}", res);

                let res = res.unwrap();
                assert_eq!(res, $($tt)*);
            }
        };
    }
    parse_test!(empty_annotation test that "{}" parses with parse_annotations);
    parse_test!(single_annotation test that "{no-layout}" parses with parse_annotations);
    parse_test!(multiple_annotation test that "{no-layout, no-layout}" parses with parse_annotations);
    parse_test!(trailing_comma_annotation test that "{no-layout, }" parses with parse_annotations);
    parse_test!(double_trailing_comma_annotation test that "{no-layout,, }" fails to parse with parse_annotations);
    parse_test!(leading_comma_annotation test that "{,no-layout}" fails to parse with parse_annotations);
    parse_test!(layout_annotation test that "   {  no-layout  ,  no-layout  , }  " parses with parse_annotations);

    parse_test!(simple_cc test that "[]" parses with parse_character_class);
    parse_test!(simple_inversion_cc test that "[^]" parses with parse_character_class);
    parse_test!(with_chars_cc test that "[abc]" parses with parse_character_class);
    parse_test!(with_range_cc test that "[a-z]" parses with parse_character_class);
    parse_test!(with_ranges_cc test that "[a-z0-9]" parses with parse_character_class);
    parse_test!(no_end_range_cc test that "[a-]" fails to parse with parse_character_class);
    parse_test!(inverted_range_cc test that "[z-a]" fails to parse with parse_character_class);

    parse_test!(string test that "'test'" parses with parse_literal to "test");
    parse_test!(string_dq test that "\"test\"" parses with parse_literal to "test");
    parse_test!(string_backslash test that "\"te\\\\st\"" parses with parse_literal to "te\\st");
    parse_test!(string_escaped_quote test that "\"te\\\"st\"" parses with parse_literal to "te\"st");
    parse_test!(string_no_matching_quotes test that "\"test'" fails to parse with parse_literal);

    parse_test!(simple_sort test that "a = 'test'" parses with parse_sort_or_meta);
    parse_test!(two_constructor_sort test that "a = 'test' | 'test'" parses with parse_sort_or_meta);
    parse_test!(repeat_0_n test that "a = x*" parses with parse_sort_or_meta);
    parse_test!(repeat_1_n test that "a = x+" parses with parse_sort_or_meta);
    parse_test!(repeat_0_1 test that "a = x?" parses with parse_sort_or_meta);
    parse_test!(repeat_x_y test that "a = x{3, 5}" parses with parse_sort_or_meta);
    parse_test!(repeat_x test that "a = x{3}" parses with parse_sort_or_meta);

    parse_test!(integration_1 test that r#"
    char = [0-9];
    string = "\"" char* "\"";

    start at string;
    "# parses with parse_file);

    parse_test!(integration_2 test that r#"
C_=_ "v" _Efm{9,2}|((W5{1} (((_*)? W _) "u"+ ("c")+)*){47,4}){3888};

layout = [^Q-j];
layout = [^T-o];
start at r8;
    "# parses with parse_file);

    macro_rules! character_class_test {
        ($name: ident cc $input: literal contains $($c:literal)*) => {
            #[test]
            fn $name () {
                let sf = SourceFile::new_for_test($input);
                let mut sfi = sf.iter();
                let res = parse_character_class(&mut sfi);
                assert!(res.is_ok(), "{:?}", res);
                let res = res.unwrap();

                $(
                    for c in $c.chars() {
                        assert!(res.contains(c));
                    }
                )*
            }
        };
        ($name: ident cc $input: literal excludes $($c:literal)*) => {
            #[test]
            fn $name () {
                let sf = SourceFile::new_for_test($input);
                let mut sfi = sf.iter();
                let res = parse_character_class(&mut sfi);
                assert!(res.is_ok(), "{:?}", res);
                let res = res.unwrap();

                $(
                    for c in $c.chars() {
                        assert!(!res.contains(c), "{:?}", res);
                    }
                )*
            }
        };
    }

    character_class_test!(simple cc "[a-z]" contains "abcdefghijklmnopqrstuvwxyz");
    character_class_test!(simple_exclude cc "[a-z]" excludes "0123456789ABCDEFHIJKLMNOPQRSTUVWXYZ:)");
    character_class_test!(multiple_groups cc "[a-z0-9]" contains "abcdefghijklmnopqrstuvwxyz0123456789");
    character_class_test!(inverted_cc cc "[^a-z0-9]" excludes "abcdefghijklmnopqrstuvwxyz0123456789");
    character_class_test!(inverted_cc_2 cc "[^a-z0-9]" contains "ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    character_class_test!(just_some_chars cc "[abc]" contains "abc");
    character_class_test!(just_some_chars_2 cc "[abc]" excludes "xyz");

    const GRAMMAR: &str = r##"
<program> ::= <rulelist> <starting>
<rulelist> ::= <rule-or-meta> <rulelist> | ""

<09> ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
<number> ::= <09> <number> | <09>

<az> ::= "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l" | "m" | "n" | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v" | "w" | "x" | "y" | "z"
<AZ> ::= "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J" | "K" | "L" | "M" | "N" | "O" | "P" | "Q" | "R" | "S" | "T" | "U" | "V" | "W" | "X" | "Y" | "Z"

<name-char> ::= <az> | <AZ> | "_"
<name-end-char> ::= <name-char> | <09>

<name-end> ::= <name-end-char> <name-end> | <name-end-char>
<name> ::= <name-char> <name-end> | <name-char>

<range> ::= <name-char> "-" <name-char>
<charclassitem> ::= <name-char> | <range> | ""
<charclass> ::= "[" <charclassitem> "]" | "[^" <charclassitem> "]"

<starting> ::= "start at " <name> ";"
<meta> ::= "layout = " <charclass>

<literal> ::= '"' <name> '"' | "'" <name> "'"

<suffix> ::= "*" | "+" | "?" | "{" <number> "}" | "{" <number> "," <number> "}"

<constructor-atom> ::= <name> | <literal> | "(" <constructor> ")"
<constructor-without-suffix> ::= <constructor-atom>
<simple-constructor> ::= <constructor-without-suffix> | <constructor-without-suffix> <suffix>
<constructor> ::= <simple-constructor> | <simple-constructor> " " <constructor>

<rule> ::= <constructor> | <constructor> "|" <constructor>
<sort> ::= <name> "=" <rule>
<rule-or-meta> ::= <sort> ";" | <meta> ";"


    "##;

    fn generate_sentence(g: &Grammar) -> String {
        loop {
            let res = g.generate_callback(|ident, value| match ident {
                "range" => {
                    // make sure ranges have a left side smaller than their right side
                    let parts: Vec<_> = value.split("-").collect();
                    let (p1, p2) = (parts[0], parts[1]);
                    assert_eq!(p1.chars().count(), 1);
                    assert_eq!(p1.chars().count(), 1);

                    (p1.chars().next().unwrap() as u32) < (p2.chars().next().unwrap() as u32)
                }
                _ => true,
            });
            match res {
                Ok(i) => break i,
                Err(bnf::Error::RecursionLimit(_)) => continue,
                _ => panic!("aaaaa"),
            }
        }
    }

    #[test]
    pub fn fuzz() {
        let grammar: Grammar = match GRAMMAR.parse() {
            Ok(i) => i,
            Err(e) => {
                panic!("{:#?}", e);
            }
        };

        for _ in 0..1000 {
            let sentence = generate_sentence(&grammar);
            let file = SourceFile::new_for_test(&sentence);

            if let Err(e) = parse(&file) {
                panic!("failed on program: {sentence}: {e:?}");
            }
        }
    }
}

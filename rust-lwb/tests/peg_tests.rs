use miette::GraphicalReportHandler;
use rust_lwb::language::Language;
use rust_lwb::parser::peg::parser_sugar::parse_file;
use rust_lwb::parser::syntax_file::convert_syntax_file_ast::convert;
use rust_lwb::parser::syntax_file::SyntaxFile;
use rust_lwb::sources::source_file::SourceFile;

macro_rules! peg_test {
    (name: $name:ident, syntax: $syntax:literal, passing tests: $($input_pass:literal)* failing tests: $($input_fail:literal)*) => {
        #[test]
        fn $name() {
            let sf = SourceFile::new($syntax.to_string(), "test.syntax".to_string());
            let ast = SyntaxFile::parse(&sf);
            let ast = convert(ast).unwrap();

            $(
            println!("== Parsing (should be ok): {}", $input_pass);
            let sf2 = SourceFile::new($input_pass.to_string(), "input.language".to_string());
            let (parsed, errs) = parse_file(&ast, &sf2);
            println!("{}", parsed);
            for err in &errs {
                println!("{:?}", err);
                if $input_pass != "" {
                    let mut s = String::new();
                    GraphicalReportHandler::new()
                        .with_links(true)
                       .render_report(&mut s, err)
                        .unwrap();
                    print!("{}", s);
                }
            }
            if errs.len() > 0 { assert!(false); }
            )*

            $(
            println!("== Parsing (should be err): {}", $input_fail);
            let sf2 = SourceFile::new($input_fail.to_string(), "input.language".to_string());
            let (parsed, errs) = parse_file(&ast, &sf2);
            println!("{}", parsed);
            for err in &errs {
                println!("{:?}", err);
                if $input_fail != "" {
                    let mut s = String::new();
                    GraphicalReportHandler::new()
                        .with_links(true)
                        .render_report(&mut s, err)
                        .unwrap();
                    print!("{}", s);
                }
            }
            if errs.len() == 0 { assert!(false); }
            )*
        }
    };
}

peg_test! {
name: as_rightrec,
syntax: r#"
As:
    More = "a" As;
    NoMore = "";
start at As;
"#,
passing tests:
    ""
    "a"
    "aa"
    "aaa"
failing tests:
    "b"
    "ab"
    "ba"
    "aac"
}

peg_test! {
name: as_leftrec,
syntax: r#"
As:
    More = As "a";
    NoMore = "";
start at As;
"#,
passing tests:
    ""
    "a"
    "aa"
    "aaa"
failing tests:
    "b"
    "ab"
    "ba"
    "aac"
}

peg_test! {
name: bad_leftrec,
syntax: r#"
X = X;
start at X;
"#,
passing tests:
failing tests:
    ""
    "a"
    "aa"
    "aaa"
    "aaaa"
}

peg_test! {
name: bad_loop,
syntax: r#"
X = ""*;
start at X;
"#,
passing tests:
failing tests:
    "a"
    "aa"
    "aaa"
    "aaaa"
    ""
}

peg_test! {
name: layout,
syntax: r#"
X = "x" "y";
layout = [\n\r\t ];
start at X;
"#,
passing tests:
    "x y"
    "xy"
failing tests:
    "x"
}

peg_test! {
name: layout_in_layout,
syntax: r#"
X = "x" "y";
layout = "a" "b";
start at X;
"#,
passing tests:
    "xy"
    "xaby"
    "xababy"
failing tests:
    "x"
    "xa by"
    "xaabby"
}

peg_test! {
name: no_layout,
syntax: r#"
X = "x" "y"; {no-layout}
layout = [\n\r\t ];
start at X;
"#,
passing tests:
    "xy"
failing tests:
    "x y"
    "x_y"
    "x
y"
}

peg_test! {
name: simple,
syntax: r#"
X = "x";
start at X;
"#,
passing tests:
    "x"
failing tests:
}

peg_test! {
name: recovery1,
syntax: r#"
X = "x"+ ";";
XS = X*;
start at XS;
"#,
passing tests:
    "x;"
    "xx;"
    "xx;x;"
    "x;xx;x;xxx;"
failing tests:
    "x"
    "xx"
    "x;x"
    "xx;;"
    ";"
}

peg_test! {
name: recovery2,
syntax: r#"
S = ("{" ("x"+ ";")* "}")*;
start at S;
"#,
passing tests:
failing tests:
    "{x;{x;}"
    "{x;{x;{x;}"
    "{x;;x;;x;}"
    "{x;x}{x;}"
}

peg_test! {
name: recovery3,
syntax: r#"
S = "0" "1" "2" "3" "4" "5" "6" "7" "8" "9";
start at S;
"#,
passing tests:
failing tests:
    "0234x679"
    "0234679"
    "0134569"
}

peg_test! {
name: comments,
syntax: r#"
T = "x"+ ";"; {no-layout}
S = ("{" T* "}")*;
layout = "/*" [a-zA-Z]* "*/";
start at S;
"#,
passing tests:
    "{x;x;}"
    "{x;/*comment*/x;}"
    "{x;x;}/*comment*/"
    "/*comment*/{x;x;}"
failing tests:
    "{x/*comment*/;x;}"
}

peg_test! {
name: exact_repetition,
syntax: r#"
S = "a"{3};
start at S;
"#,
passing tests:
    "aaa"
failing tests:
    "aa"
    "aaaa"
    ""
}

peg_test! {
name: ranged_repetition,
syntax: r#"
S = "a"{3, 5};
start at S;
"#,
passing tests:
    "aaa"
    "aaaa"
    "aaaaa"
failing tests:
    "aa"
    "aaaaaa"
    ""
}

peg_test! {
name: unbounded_repetition,
syntax: r#"
S = "a"{3, inf};
start at S;
"#,
passing tests:
    "aaa"
    "aaaa"
    "aaaaa"
    "aaaaaa"
    "aaaaaaa"
    "aaaaaaaa"
    "aaaaaaaaa"
    "aaaaaaaaaa"
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
failing tests:
    "aa"
    ""
}

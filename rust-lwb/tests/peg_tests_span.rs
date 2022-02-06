use rust_lwb::codegen_prelude::ParsePairExpression;
use rust_lwb::language::Language;
use rust_lwb::parser::peg::parser_sugar::parse_file;
use rust_lwb::parser::syntax_file::convert_syntax_file_ast::convert;
use rust_lwb::parser::syntax_file::SyntaxFile;
use rust_lwb::sources::source_file::SourceFile;

#[test]
fn test_spans() {
    let syntax = r#"
expression:
    add = "true" "++" expression;
    sub = [0-9]+ "-" [0-9]+;

    paren = "(" expression ")";

program:
    program = expression ";";

start at program;
layout:
    layout = [\n\r\t ];

    "#;

    let input = "  true ++ ( 13 - 5 ) ;  ";

    let sf = SourceFile::new(syntax.to_string(), "test.syntax".to_string());
    let ast = match SyntaxFile::parse(&sf) {
        Ok(ok) => ok,
        Err(err) => {
            println!("{}", err);
            panic!();
        }
    };
    let ast = convert(ast).unwrap();

    let sf2 = SourceFile::new(input.to_string(), "input.language".to_string());
    let (parsed, errs) = parse_file(&ast, &sf2);
    assert_eq!(errs.len(), 0);

    assert_eq!(parsed.span().position, 2);
    assert_eq!(parsed.span().length, 20);

    //Statement
    let p1 = if let ParsePairExpression::List(s, l) = parsed.constructor_value {
        (s, l)
    } else {
        unreachable!()
    };
    assert_eq!(p1.0.position, 2);
    assert_eq!(p1.0.length, 20);

    //Expression sort
    let p2 = if let ParsePairExpression::Sort(s, l) = &p1.1[0] {
        (s, l)
    } else {
        unreachable!()
    };
    assert_eq!(p2.0.position, 2);
    assert_eq!(p2.0.length, 18);

    //Semicolon
    let p3 = if let ParsePairExpression::Empty(s) = &p1.1[1] {
        s
    } else {
        unreachable!()
    };
    assert_eq!(p3.position, 21);
    assert_eq!(p3.length, 1);

    //Expression
    let p4 = if let ParsePairExpression::List(s, l) = &p2.1.constructor_value {
        (s, l)
    } else {
        unreachable!()
    };
    assert_eq!(p4.0.position, 2);
    assert_eq!(p4.0.length, 18);

    //True
    let p5 = if let ParsePairExpression::Empty(s) = &p4.1[0] {
        s
    } else {
        unreachable!()
    };
    assert_eq!(p5.position, 2);
    assert_eq!(p5.length, 4);

    //++
    let p6 = if let ParsePairExpression::Empty(s) = &p4.1[1] {
        s
    } else {
        unreachable!()
    };
    assert_eq!(p6.position, 7);
    assert_eq!(p6.length, 2);

    //( 13 - 5 ) sort
    let p7 = if let ParsePairExpression::Sort(s, l) = &p4.1[2] {
        (s, l)
    } else {
        unreachable!()
    };
    assert_eq!(p7.0.position, 10);
    assert_eq!(p7.0.length, 10);

    //( 13 - 5 )
    let p8 = if let ParsePairExpression::List(s, l) = &p7.1.constructor_value {
        (s, l)
    } else {
        unreachable!()
    };
    assert_eq!(p7.0.position, 10);
    assert_eq!(p7.0.length, 10);
    assert_eq!(p8.1[0].span().position, 10);
    assert_eq!(p8.1[0].span().length, 1);
    assert_eq!(p8.1[1].span().position, 12);
    assert_eq!(p8.1[1].span().length, 6);
    assert_eq!(p8.1[2].span().position, 19);
    assert_eq!(p8.1[2].span().length, 1);

    //13 - 5
    let p9 = if let ParsePairExpression::Sort(s, l) = &p8.1[1] {
        (s, l)
    } else {
        unreachable!()
    };
    let p9 = if let ParsePairExpression::List(s, l) = &p9.1.constructor_value {
        (s, l)
    } else {
        unreachable!()
    };
    assert_eq!(p9.1[0].span().position, 12);
    assert_eq!(p9.1[0].span().length, 2);
    assert_eq!(p9.1[1].span().position, 15);
    assert_eq!(p9.1[1].span().length, 1);
    assert_eq!(p9.1[2].span().position, 17);
    assert_eq!(p9.1[2].span().length, 1);
}

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
name: delimited_star,
syntax: r#"
X = delimited("x", ",", *);
start at X;
"#,
passing tests:
    ""
    "x"
    "x,x"
    "x,x,x"
failing tests:
    "x,"
    "x,x,"
    ","
    "x,,x"
    "xx,x"
}

peg_test! {
name: delimited_plus,
syntax: r#"
X = delimited("x", ",", +);
start at X;
"#,
passing tests:
    "x"
    "x,x"
    "x,x,x"
failing tests:
    ""
    "x,"
    "x,x,"
    ","
    "x,,x"
    "xx,x"
}

peg_test! {
name: delimited_least_two,
syntax: r#"
X = delimited("x", ",", 2, inf);
start at X;
"#,
passing tests:
    "x,x"
    "x,x,x"
failing tests:
    ""
    "x"
    "x,"
    "x,x,"
    ","
    "x,,x"
    "xx,x"
}

peg_test! {
name: delimited_most_two,
syntax: r#"
X = delimited("x", ",", 0, 2);
start at X;
"#,
passing tests:
    ""
    "x"
    "x,x"
failing tests:
    "x,x,x"
    "x,"
    "x,x,"
    ","
    "x,,x"
    "xx,x"
}

peg_test! {
name: delimited_trailing_star,
syntax: r#"
X = delimited("x", ",", *, trailing);
start at X;
"#,
passing tests:
    ""
    "x"
    "x,x"
    "x,"
    "x,x,"
    "x,x,x"
    ","
failing tests:
    "x,,x"
    "xx,x"
}

peg_test! {
name: delimited_trailing_plus,
syntax: r#"
X = delimited("x", ",", +, trailing);
start at X;
"#,
passing tests:
    "x"
    "x,x"
    "x,"
    "x,x,"
    "x,x,x"
failing tests:
    ""
    ","
    "x,,x"
    "xx,x"
}

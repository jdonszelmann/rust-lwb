use miette::GraphicalReportHandler;
use rust_lwb::parser::peg::parser::parse_file;
use rust_lwb::sources::source_file::SourceFile;
macro_rules! peg_test {
    (name: $name:ident, syntax: $syntax:literal, passing tests: $($input_pass:literal)* failing tests: $($input_fail:literal)*) => {
        #[test]
        fn $name() {
            let sf = SourceFile::new($syntax.to_string(), "test.syntax".to_string());
            let ast = rust_lwb::parser::bootstrap::parse(&sf).unwrap();

            $(
            let sf2 = SourceFile::new($input_pass.to_string(), "input.language".to_string());
            let parsed = parse_file(&ast, &sf2);
            match parsed {
                Ok(_) => {}
                Err(err) => {
                    println!("Unexpected error while parsing: {}", $input_pass);
                    let mut s = String::new();
                    GraphicalReportHandler::new()
                        .with_links(true)
                        .render_report(&mut s, &err)
                        .unwrap();
                    print!("{}", s);
                    assert!(false);
                }
            }
            )*

            $(
            let sf2 = SourceFile::new($input_fail.to_string(), "input.language".to_string());
            let parsed = parse_file(&ast, &sf2);
            match parsed {
                Ok(ok) => {
                    println!("Unexpected ok while parsing: {}", $input_fail);
                    print!("{:?}", ok);
                    assert!(false);
                }
                Err(_) => {}
            }
            )*
        }
    };
}

peg_test! {
name: as_rightrec,
syntax: r#"
As:
    More = 'a' As;
    NoMore = '';
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

// peg_test! {
// name: as_leftrec,
// syntax: r#"
// As:
//     More = As 'a';
//     NoMore = '';
// start at As;
// "#,
// passing tests:
//     ""
//     "a"
//     "aa"
//     "aaa"
// failing tests:
//     "b"
//     "ab"
//     "ba"
//     "aac"
// }

peg_test! {
name: actual_leftrec,
syntax: r#"
X:
    Fail = X;
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

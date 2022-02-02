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
            println!("== Parsing (should be ok): {}", $input_pass);
            let sf2 = SourceFile::new($input_pass.to_string(), "input.language".to_string());
            let parsed = parse_file(&ast, &sf2);
            match parsed {
                Ok(ok) => {
                    println!("{:?}", ok);
                }
                Err(err) => {
                    println!("{:?}", err);
                    if $input_pass != "" {
                        let mut s = String::new();
                        GraphicalReportHandler::new()
                            .with_links(true)
                            .render_report(&mut s, &err)
                            .unwrap();
                        print!("{}", s);
                    }
                }
            }
            )*

            $(
            println!("== Parsing (should be err): {}", $input_fail);
            let sf2 = SourceFile::new($input_fail.to_string(), "input.language".to_string());
            let parsed = parse_file(&ast, &sf2);
            match parsed {
                Ok(ok) => {
                    println!("{:?}", ok);
                    assert!(false);
                }
                Err(err) => {
                    println!("{:?}", err);
                    if $input_fail != "" {
                        let mut s = String::new();
                        GraphicalReportHandler::new()
                            .with_links(true)
                            .render_report(&mut s, &err)
                            .unwrap();
                        print!("{}", s);
                    }
                }
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

peg_test! {
name: as_leftrec,
syntax: r#"
As:
    More = As 'a';
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

peg_test! {
name: bad_leftrec,
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

peg_test! {
name: bad_loop,
syntax: r#"
X:
    Fail = ''*;
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
name: recovery,
syntax: r#"
X:
    X = 'x'+ ';';
XS:
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
name: layout,
syntax: r#"
X:
    X = 'x' 'y';
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
name: no_layout,
syntax: r#"
X:
    X = 'x' 'y'; {no-layout}
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

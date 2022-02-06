use itertools::Itertools;
use rust_lwb::language::Language;
use rust_lwb::parser::peg::parser_file::parse_file;
use rust_lwb::parser::syntax_file::convert_syntax_file_ast::convert;
use rust_lwb::parser::syntax_file::SyntaxFile;
use rust_lwb::sources::source_file::SourceFile;

macro_rules! peg_test_recover {
    ($name:ident, $syntax:literal, $($input:literal : $positions:expr),+) => {
        #[test]
        fn $name() {
            let sf = SourceFile::new($syntax.to_string(), "test.syntax".to_string());
            let ast = convert(SyntaxFile::parse(&sf).unwrap()).unwrap();

            $(
            let sf2 = SourceFile::new($input.to_string(), "input.language".to_string());
            let (_, errs) = parse_file(&ast, &sf2);
            assert_eq!(errs.iter().map(|e| e.span.position).collect_vec(), $positions);
            )+



        }
    }
}

peg_test_recover! {
    sequence,
    r#"
program:
    program = "a" "b" "c" "d";
start at program;
    "#,
    "" : [0],
    "a" : [1],
    "b" : [0],
    "abd" : [2],
    "abc" : [3],
    "ad" : [1]
}

peg_test_recover! {
    recovery_lang,
    r#"
program:
    program = ("{" ("x"+ ";")* "}")*;
start at program;
    "#,
    "{x;{x;}" : [3],
    "{x;{x;{x;}" : [3,6],
    "{x;;x;;x;}" : [3,6],
    "{x;x}{x;}" : [4]
}

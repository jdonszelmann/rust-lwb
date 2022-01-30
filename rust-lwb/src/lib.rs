
/// Helpers for type checking
pub mod types;

/// Helpers for code generation
pub mod transform;

pub mod syntax_file;


mod tests {
    use rust_lwb_parser::parser::syntax_file::parse;
    use rust_lwb_parser::source_file::SourceFile;

    #[test]
    fn test_parse() {
        let sf = SourceFile::open("src/syntax-file.syntax").unwrap();


        let ast = parse(&sf).unwrap();

    }
}

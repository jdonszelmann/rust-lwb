use miette::GraphicalReportHandler;
use rust_lwb::parser::peg::parser_file::parse_file;
use rust_lwb::parser::syntax_file::convert_syntax_file_ast::convert;
use rust_lwb::parser::syntax_file::SyntaxFile;
use rust_lwb::sources::source_file::SourceFile;

#[test]
fn test_example() {
    let syntax = r#"
S:
    S = "0" "1" "2" "3" "4" "5" "6" "7" "8" "9";
start at S;
    "#;

    let input = "0234x679";

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
    println!("{}", parsed);
    for err in &errs {
        println!("{:?}", err);
        if input != "" {
            let mut s = String::new();
            GraphicalReportHandler::new()
                .with_links(true)
                .render_report(&mut s, err)
                .unwrap();
            print!("{}", s);
        }
    }
}

use miette::GraphicalReportHandler;
use rust_lwb::parser::peg::parser::parse_file;
use rust_lwb::sources::source_file::SourceFile;

#[test]
pub fn run_example() {
    let sf = SourceFile::new(
        r#"
program:
    program = group*;
group:
    group = '{' line+ '}';
line:
    line = [a-z]+ ';';
start at program;"#
            .to_string(),
        "test.syntax".to_string(),
    );
    let ast = rust_lwb::parser::bootstrap::parse(&sf).unwrap();

    let sf = SourceFile::new("{ab;ab;}{ab;ab;}".to_string(), "".to_string());
    match parse_file(&ast, &sf) {
        Ok(ok) => {
            println!("OK: {}", ok);
        }
        Err(err) => {
            let mut s = String::new();
            GraphicalReportHandler::new()
                .with_links(true)
                .render_report(&mut s, &err)
                .unwrap();
            print!("{}", s);
        }
    }
}

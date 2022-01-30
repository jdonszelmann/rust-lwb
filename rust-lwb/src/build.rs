use std::error::Error;
use rust_lwb_parser::source_file::SourceFile;
use rust_lwb_parser::parser::syntax_file::parse;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=src/syntax-file.syntax");


    let sf = SourceFile::open("src/syntax-file.syntax")?;


    let ast = parse(&sf)?;
    Ok(())
}
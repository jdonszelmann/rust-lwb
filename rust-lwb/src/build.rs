use std::error::Error;
use std::io::Write;
use rust_lwb_parser::source_file::SourceFile;
use rust_lwb_parser::parser::syntax_file::parse;
use rust_lwb_parser::codegen::codegen::generate_language;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=src/syntax-file.syntax");


    let sf = SourceFile::open("src/syntax-file.syntax")?;
    let ast = parse(&sf)?;

    let res = generate_language(ast);

    let mut res_file = std::fs::File::create("src/syntax_file.rs")?;
    res_file.write_all(res.as_bytes())?;

    Ok(())
}
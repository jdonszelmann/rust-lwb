use rust_lwb::codegen::generate_language;
use rust_lwb::parser::bootstrap::parse;
use rust_lwb::sources::source_file::SourceFile;
use std::error::Error;
use std::io::Write;

/// Contains code related to parsing syntax
/// definition files

fn main() -> Result<(), Box<dyn Error>> {
    let sf = SourceFile::open("src/syntax-file.syntax")?;
    let ast = parse(&sf)?;

    let res = generate_language(ast);

    let mut res_file = std::fs::File::create("src/syntax_file.rs")?;
    res_file.write_all(res.as_bytes())?;

    Ok(())
}

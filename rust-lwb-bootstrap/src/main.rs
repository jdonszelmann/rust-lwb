mod syntax_file;

use std::error::Error;
use rust_lwb::codegen::manager::CodegenManager;

fn main() -> Result<(), Box<dyn Error>> {
    let mut m = CodegenManager::new();
    m.add_syntax_file("rust-lwb-bootstrap/syntax-file.syntax")
        .destination("rust-lwb-bootstrap/src/syntax_file.rs");

    m.codegen()?;

    Ok(())
}

use rust_lwb::codegen::manager::CodegenManager;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut m = CodegenManager::new();
    m.add_syntax_file("rust-lwb-bootstrap/syntax-file.syntax")
        .import_location("crate")
        .destination("rust-lwb/src/parser/syntax_file/ast.rs");
    m.codegen()?;

    Ok(())
}

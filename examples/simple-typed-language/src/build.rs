use rust_lwb::codegen::manager::CodegenManager;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut m = CodegenManager::new();

    m.add_syntax_file("src/stl.syntax");

    m.codegen()?;

    Ok(())
}

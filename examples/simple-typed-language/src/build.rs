use std::error::Error;
use rust_lwb::codegen::manager::CodegenManager;

fn main() -> Result<(), Box<dyn Error>>{
    let mut m = CodegenManager::new();

    m.add_syntax_file("src/stl.syntax");

    m.codegen()?;

    Ok(())
}
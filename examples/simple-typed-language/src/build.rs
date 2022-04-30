use rust_lwb::codegen::manager::Codegen;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let m = Codegen::new()?;
    m.codegen()?;

    Ok(())
}

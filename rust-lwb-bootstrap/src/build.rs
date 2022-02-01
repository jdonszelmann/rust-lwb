mod bootstrap_config;

use std::error::Error;
use rust_lwb::codegen::manager::CodegenManager;
use crate::bootstrap_config::from_root;

fn main() -> Result<(), Box<dyn Error>> {
    let config = bootstrap_config::load("bootstrap.toml");

    let temporary_language_location = from_root("src/temp.rs");

    let mut m = CodegenManager::new();
    m.add_syntax_file(from_root(config.input_location))
        .import_location("crate")
        .destination(temporary_language_location)
        .serde(true);

    m.codegen()?;

    Ok(())
}

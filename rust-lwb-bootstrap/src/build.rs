extern crate core;

mod bootstrap_config;

use crate::bootstrap_config::{from_root, temporary_location, unwrap};
use rust_lwb::codegen::manager::CodegenManager;

fn main() {
    let config = bootstrap_config::load("bootstrap.toml");

    let temporary_language_location = temporary_location();

    let mut m = CodegenManager::new();
    m.add_syntax_file(from_root(config.input_location))
        .import_location("crate")
        .destination(temporary_language_location)
        .serde(true)
        .dont_generate_serialized_ast();


    unwrap(m.codegen());
}

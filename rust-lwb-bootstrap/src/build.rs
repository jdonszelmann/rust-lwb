extern crate core;

mod bootstrap_config;

use crate::bootstrap_config::{from_root, temporary_location, unwrap};
use rust_lwb::codegen::manager::Codegen;
use rust_lwb::config::{Config, LanguageConfig, SyntaxConfig};

fn main() {
    let config = bootstrap_config::load("bootstrap.toml");

    let temporary_language_location = temporary_location();

    unwrap(
        Codegen::with_config_struct(Config {
            syntax: SyntaxConfig {
                destination: temporary_language_location.to_string_lossy().into_owned(),
                definition: from_root(config.input_location)
                    .to_string_lossy()
                    .into_owned(),
                non_exhaustive: false,
                serde: true,
                import_location: "crate".to_string(),
                write_serialized_ast: false,
            },
            language: LanguageConfig {
                name: "syntax definition file".to_string(),
                extensions: vec![],
            },
        })
        .codegen(),
    );
}

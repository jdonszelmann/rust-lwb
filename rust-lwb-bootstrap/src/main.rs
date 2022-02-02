use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use miette::GraphicalReportHandler;
use rust_lwb::parser::bootstrap::parse;
use rust_lwb::sources::source_file::SourceFile;
use rust_lwb::parser::ast::generate_ast::generate_ast;
use rust_lwb::parser::peg::parser::parse_file;
use crate::bootstrap_config::from_root;

mod bootstrap_config;

mod codegen_prelude {
    pub use rust_lwb::codegen_prelude::*;
}

mod temp;

fn main() -> Result<(), Box<dyn Error>> {
    let config = bootstrap_config::load("bootstrap.toml");

    // parse the syntax definition again with the old parse
    println!("parsing {}", config.input_location);
    let sf = SourceFile::open(from_root(&config.input_location))?;
    let ast = parse(&sf)?; // TODO: replace with bootstrapped parser

    println!("reparsing {} with peg parser and output from previous parse", config.input_location);
    let bootstrapped_syntax_file_ast_pairs = match parse_file(&ast, &sf) {
        Ok(i) => {
            // println!("{}", i);
            i
        },
        Err(e) => {
            let mut s = String::new();
            GraphicalReportHandler::new()
                .with_links(true)
                .render_report(&mut s, &e)
                .unwrap();
            panic!("{}", s);
        }
    };
    println!("generating ast from pairs");
    let bootstrapped_syntax_file_ast: temp::AST_ROOT<_> = generate_ast(&bootstrapped_syntax_file_ast_pairs);

    println!("serializing ast");
    let serialized_ast = rust_lwb::bincode::serialize(&bootstrapped_syntax_file_ast)?;

    println!("writing serialized ast at {}", config.serialized_ast_location);
    let mut res_file = std::fs::File::create(from_root(config.serialized_ast_location))?;
    res_file.write_all(&serialized_ast)?;

    println!("creating backup of {}", config.output_location);
    // if everything went well, replace the old ast types with the new ast types
    let mut backup = config.output_location.clone();
    backup.push_str(".backup");
    std::fs::rename(from_root(&config.output_location), from_root(backup))?;

    println!("moving ast types to {}", config.output_location);
    std::fs::rename(from_root("src/temp.rs"), from_root(config.output_location))?;

    Ok(())
}

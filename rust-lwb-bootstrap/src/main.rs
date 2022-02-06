use miette::GraphicalReportHandler;
use rust_lwb::language::Language;
use rust_lwb::parser::ast::generate_ast::generate_ast;
use rust_lwb::parser::peg::parser_core_file::parse_file;
use rust_lwb::sources::source_file::SourceFile;
use std::error::Error;
use std::io::Write;
// use rust_lwb::parser::bootstrap::parse;
use crate::bootstrap_config::{from_root, temporary_location, unwrap};
use rust_lwb::parser::syntax_file::convert_syntax_file_ast::convert;
use rust_lwb::parser::syntax_file::SyntaxFile;

mod bootstrap_config;

mod codegen_prelude {
    pub use rust_lwb::codegen_prelude::*;
}

#[rustfmt::skip]
mod temp;

fn main() -> Result<(), Box<dyn Error>> {
    let config = bootstrap_config::load("bootstrap.toml");

    // parse the syntax definition again with the old parse
    println!("parsing {}", config.input_location);
    let sf = SourceFile::open(from_root(&config.input_location))?;
    let ast = unwrap(SyntaxFile::parse(&sf));
    let legacy_ast = unwrap(convert(ast));

    println!(
        "reparsing {} with peg parser and output from previous parse",
        config.input_location
    );
    let (bootstrapped_syntax_file_ast_pairs, errs) = parse_file(&legacy_ast, &sf);
    let mut s = String::new();
    for err in &errs {
        GraphicalReportHandler::new()
            .with_links(true)
            .render_report(&mut s, err)
            .unwrap();
    }
    if !errs.is_empty() {
        panic!("{}", s);
    }

    println!("generating ast from pairs");
    let bootstrapped_syntax_file_ast: temp::AST_ROOT<_> =
        generate_ast(&bootstrapped_syntax_file_ast_pairs);

    println!("serializing ast");
    let serialized_ast = rust_lwb::bincode::serialize(&bootstrapped_syntax_file_ast)?;

    println!("creating backup of {}", config.output_location);
    // if everything went well, replace the old ast types with the new ast types
    let mut backup = config.output_location.clone();
    backup.push_str(".backup");
    std::fs::remove_dir_all(from_root(&backup))?;
    std::fs::rename(from_root(&config.output_location), from_root(backup))?;

    println!("appending serialized ast");
    {
        let mut serialized_parser_path = temporary_location();
        serialized_parser_path.push("parser.rs");

        let mut file = std::fs::File::options()
            .append(true)
            .open(serialized_parser_path)?;
        file.write_all(
            format!(r##"pub const PARSER: &[u8] = &{:?};"##, serialized_ast).as_bytes(),
        )?;
    }

    println!("moving ast types to {}", config.output_location);
    std::fs::rename(temporary_location(), from_root(config.output_location))?;

    Ok(())
}

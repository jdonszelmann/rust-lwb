use std::error::Error;
use std::io::Write;
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
    let sf = SourceFile::open(from_root(config.input_location))?;
    let ast = parse(&sf)?; // TODO: replace with bootstrapped parser

    let bootstrapped_syntax_file_ast_pairs = match parse_file(&ast, &sf) {
        Ok(i) => i,
        Err(e) => {
            let mut s = String::new();
            GraphicalReportHandler::new()
                .with_links(true)
                .render_report(&mut s, &e)
                .unwrap();
            panic!("{}", s);
        }
    };
    let bootstrapped_syntax_file_ast: temp::AST_ROOT<_> = generate_ast(&bootstrapped_syntax_file_ast_pairs);

    let serialized_ast = rust_lwb::bincode::serialize(&bootstrapped_syntax_file_ast)?;

    let mut res_file = std::fs::File::create(from_root(config.serialized_ast_location))?;
    res_file.write_all(&serialized_ast)?;

    Ok(())
}

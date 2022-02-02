use miette::{GraphicalReportHandler, GraphicalTheme};
use crate::codegen_prelude::AstInfo;
use crate::parser::ast::AstNode;
use crate::parser::ast::generate_ast::{BasicAstInfo, BasicAstNode, generate_ast};
use crate::parser::bootstrap;
use crate::parser::peg::parser::parse_file;
use crate::parser::syntax_file::convert_syntax_file_ast::{AstConversionError, convert_syntax_file_ast};
use crate::sources::source_file::SourceFile;
use thiserror::Error;
use crate::parser::peg::parse_error::PEGParseError;

mod ast;

const SERIALIZED_AST: &[u8] = include_bytes!("serialized-ast.bin");

fn display_miette_error(err: &impl miette::Diagnostic) -> String {
    let mut s = String::new();
    GraphicalReportHandler::new()
        .with_links(true)
        .with_theme(GraphicalTheme::unicode())
        .render_report(&mut s, err)
        .unwrap();
    s
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("failed to deserialize saved syntax file definition ast (this is a bug! please report it)")]
    Bincode(#[from] bincode::Error),

    #[error("failed to convert saved syntax file definition ast to legacy syntax file definition ast (this is a bug! please report it)")]
    ConvertAstError(#[from] AstConversionError),

    #[error("parse error\n{}", display_miette_error(_0))]
    PEG(#[from] PEGParseError),
}

pub fn parse_language<AST: BasicAstNode>(input: &SourceFile) -> Result<AST, ParseError> {
    let syntax_file_ast: ast::AST_ROOT<BasicAstInfo> = bincode::deserialize_from(SERIALIZED_AST)?;
    let legacy_ast = convert_syntax_file_ast(syntax_file_ast)?;

    // TODO: inline the parser into the library (so we don't need to reparse the syntax definition file every time)
    let pairs = parse_file(&legacy_ast, input)?;

    let ast = generate_ast(&pairs);

    Ok(ast)
}

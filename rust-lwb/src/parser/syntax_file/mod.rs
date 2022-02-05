use crate::parser::ast::generate_ast::{generate_ast, BasicAstInfo, BasicAstNode};
use crate::parser::peg::parse_error::PEGParseError;
use crate::parser::peg::parser_file::parse_file;
use crate::parser::syntax_file::convert_syntax_file_ast::{convert, AstConversionError};
use crate::sources::source_file::SourceFile;
use thiserror::Error;
use crate::error::display_miette_error;
use itertools::Itertools;

#[rustfmt::skip]
pub mod ast;
pub mod convert_syntax_file_ast;

language!(pub SyntaxFile at mod ast);

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("failed to deserialize saved parser")]
    Bincode(#[from] bincode::Error),

    #[error("failed to convert saved syntax file definition ast to legacy syntax file definition ast (this is a bug! please report it)")]
    ConvertAstError(#[from] AstConversionError),

    #[error("PEG Errors: \n{}", _0.iter().map(display_miette_error).join("\n"))]
    PEG(Vec<PEGParseError>),
}

pub fn parse_language<AST: BasicAstNode>(
    input: &SourceFile,
    parser: &[u8],
) -> Result<AST, ParseError> {
    // let syntax_file_ast: ast::AST_ROOT<BasicAstInfo> = bincode::deserialize(SERIALIZED_AST).unwrap();
    // let legacy_ast = convert(syntax_file_ast)?; // TODO: make peg parser use new version of ast
    let syntax_file_ast: ast::AST_ROOT<BasicAstInfo> = bincode::deserialize(parser)?;
    let legacy_ast = convert(syntax_file_ast)?; // TODO: make peg parser use new version of ast

    // let sf = SourceFile::open("rust-lwb-bootstrap/syntax-file.syntax").expect("open error");
    // let legacy_ast = bootstrap::parse(&sf).expect("should parse");

    let (pairs, errs) = parse_file(&legacy_ast, input);
    if !errs.is_empty() {
        return Err(ParseError::PEG(errs));
    }

    let ast = generate_ast(&pairs);

    Ok(ast)
}

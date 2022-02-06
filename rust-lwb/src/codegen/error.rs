use crate::parser::syntax_file::convert_syntax_file_ast::AstConversionError;
use std::time::SystemTimeError;
use thiserror::Error;
use crate::parser::syntax_file::ParseError;

#[derive(Debug, Error)]
pub enum CodegenError {
    #[error("An io error occurred: {0}")]
    Io(#[from] std::io::Error),

    #[error("a parse error occurred: {0}")]
    ParseError(#[from] ParseError),

    #[error("failed to convert saved syntax file definition ast to legacy syntax file definition ast (this is a bug! please report it)")]
    ConvertAstError(#[from] AstConversionError),

    #[error("failed to serialize parser")]
    Bincode(#[from] bincode::Error),

    // #[error("couldn't format timestamp")]
    // Time(#[from] time::error::Format),

    #[error("filename has no extension (while creating module structure for codegen phase)")]
    NoExtension
}

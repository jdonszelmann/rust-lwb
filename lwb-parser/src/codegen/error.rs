use crate::parser::syntax_file::convert_syntax_file_ast::AstConversionError;
use crate::parser::syntax_file::ParseError;
use thiserror::Error;
use crate::parser::peg::parser_sugar_ast::SimplifyError;

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

    #[error("filename has no extension (while creating module structure for codegen phase)")]
    NoExtension,

    #[error(transparent)]
    Simplify(#[from] SimplifyError),
}

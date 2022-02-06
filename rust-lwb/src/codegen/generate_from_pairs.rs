use std::fs::File;
use crate::codegen::error::CodegenError;
use crate::parser::bootstrap::ast::SyntaxFileAst;

pub fn write_from_pairs(file: &mut File, syntax: &SyntaxFileAst) -> Result<(), CodegenError> {
    Ok(())
}
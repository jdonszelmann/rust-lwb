use crate::codegen_prelude::AstInfo;
use crate::parser::ast::AstNode;
use crate::parser::bootstrap;
use crate::parser::peg::parser::parse_file;
use crate::sources::source_file::SourceFile;

mod convert_syntax_file_ast;
mod ast;
language!(SDF at mod ast);

pub fn parse_language<M: AstInfo, AST: AstNode<M>>(input: &SourceFile) -> AST {
    todo!() // TODO: replace with bootstrapped parser
}


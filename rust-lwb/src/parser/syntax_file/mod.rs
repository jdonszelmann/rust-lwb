use crate::codegen_prelude::AstInfo;
use crate::parser::ast::AstNode;
use crate::parser::bootstrap;
use crate::parser::peg::parser::parse_file;
use crate::sources::source_file::SourceFile;

mod ast;
language!(SDF at mod ast);

pub fn convert_syntax_file_ast<M: AstInfo>(inp: ast::AST_ROOT<M>) -> bootstrap::ast::SyntaxFileAst {
    todo!()
}

pub fn parse_language<M: AstInfo, AST: AstNode<M>>(input: &SourceFile) -> AST {
    // TODO: inline the parser into the library (so we don't need to reparse the syntax definition file every time)
    // parse_file(, input)
    todo!()
}


use crate::parser::ast::from_pairs::FromPairs;
use crate::sources::span::Span;

pub mod from_pairs;
pub mod generate_ast;

pub trait SpannedAstInfo: AstInfo {
    fn span(&self) -> &Span;
}

impl AstInfo for Span {}

impl SpannedAstInfo for Span {
    fn span(&self) -> &Span {
        self
    }
}

pub trait AstInfo {}

pub trait AstNode<M: AstInfo>: FromPairs<M> {
    fn ast_info(&self) -> &M;

    fn traverse<F>(&self, _f: F)
    where
        Self: Sized,
        F: FnMut(&dyn AstNode<M>),
    {
        todo!()
    }
}

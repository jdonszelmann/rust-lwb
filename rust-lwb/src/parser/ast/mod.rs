use crate::parser::ast::from_pairs::FromPairs;
use crate::sources::span::Span;

pub mod from_pairs;
pub mod generate_ast;

pub trait SpannedAstInfo<'src>: AstInfo<'src> {
    fn span(&self) -> &Span<'src>;
}

impl<'src> AstInfo<'src> for Span<'src> {}

impl<'src> SpannedAstInfo<'src> for Span<'src> {
    fn span(&self) -> &Span<'src> {
        &self
    }
}

pub trait AstInfo<'src> {}

pub trait AstNode<'src, M: AstInfo<'src>>: FromPairs<M> {
    fn ast_info(&self) -> &M;

    fn traverse<F>(&self, _f: F)
    where
        Self: Sized,
        F: FnMut(&dyn AstNode<M>),
    {
        todo!()
    }
}


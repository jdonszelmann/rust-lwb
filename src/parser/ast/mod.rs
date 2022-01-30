use crate::span::Span;

pub trait SpannedAstInfo: AstInfo {
    fn span(&self) -> Span;
}

impl AstInfo for Span {}

impl SpannedAstInfo for Span {
    fn span(&self) -> Span {
        self.clone()
    }
}

pub trait AstInfo {}

pub trait AstNode<I: AstInfo> {
    fn ast_info(&self) -> &I;
    fn traverse<F: FnMut(&dyn AstNode<I>)>(&self, _f: F)
    where
        Self: Sized,
    {
        todo!()
    }
}

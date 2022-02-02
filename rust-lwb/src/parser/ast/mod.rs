use crate::codegen_prelude::{GenerateAstInfo, ParsePairSort};
use crate::parser::ast::from_pairs::FromPairs;
use crate::sources::span::Span;

pub mod from_pairs;
pub mod generate_ast;

pub trait SpannedAstInfo: AstInfo {
    fn span(&self) -> &Span;
}

#[derive(Hash, Eq, PartialEq)]
pub struct NodeId(usize);

pub trait AstInfo {
    fn node_id(&self) -> NodeId;
}

pub trait AstNode<M: AstInfo>: FromPairs<M> {
    fn ast_info(&self) -> &M {
        todo!()
    }

    fn traverse<F>(&self, _f: F)
    where
        Self: Sized,
        F: FnMut(&dyn AstNode<M>),
    {
        todo!()
    }
}

impl<M: AstInfo, T> FromPairs<M> for Box<T> where T: AstNode<M> {
    fn from_pairs<G: GenerateAstInfo<Result=M>>(pair: &ParsePairSort, generator: &mut G) -> Self where Self: Sized {
        Box::new(T::from_pairs(pair, generator))
    }
}

impl<M: AstInfo, T> AstNode<M> for Box<T> where T: AstNode<M> {}
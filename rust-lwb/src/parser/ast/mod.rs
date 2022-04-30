use crate::codegen_prelude::{GenerateAstInfo, ParsePairSort};
use crate::parser::ast::from_pairs::FromPairs;
use crate::sources::span::Span;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub mod from_pairs;
pub mod generate_ast;

pub trait SpannedAstInfo: AstInfo {
    fn span(&self) -> &Span;
}

#[derive(Hash, Eq, PartialEq, Copy, Clone, Serialize, Deserialize, Debug)]
pub struct NodeId(u64);

impl NodeId {
    pub(crate) fn new(value: u64) -> NodeId {
        Self(value)
    }
}

pub trait AstInfo: Debug {
    fn node_id(&self) -> NodeId;
}

pub trait AstNode<M: AstInfo>: FromPairs<M> {
    fn ast_info(&self) -> &M;
    fn sort(&self) -> &'static str;
    fn constructor(&self) -> &'static str;

    fn traverse<F>(&self, _f: F)
    where
        Self: Sized,
        F: FnMut(&dyn AstNode<M>),
    {
        todo!()
    }
}

impl<M: AstInfo, T> FromPairs<M> for Box<T>
where
    T: AstNode<M>,
{
    fn from_pairs<G: GenerateAstInfo<Result = M>>(pair: &ParsePairSort, generator: &mut G) -> Self
    where
        Self: Sized,
    {
        Box::new(T::from_pairs(pair, generator))
    }
}

impl<M: AstInfo, T> AstNode<M> for Box<T>
where
    T: AstNode<M>,
{
    fn ast_info(&self) -> &M {
        T::ast_info(self)
    }

    fn sort(&self) -> &'static str {
        T::sort(self)
    }

    fn constructor(&self) -> &'static str {
        T::constructor(self)
    }
}

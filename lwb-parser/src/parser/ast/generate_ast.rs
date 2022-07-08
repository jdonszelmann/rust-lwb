use crate::codegen_prelude::{AstInfo, GenerateAstInfo, ParsePairSort};
use crate::parser::ast::{AstNode, NodeId, SpannedAstInfo};
use crate::sources::span::Span;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct BasicAstInfo {
    span: Span,
    node_id: NodeId,
}

impl AstInfo for BasicAstInfo {
    fn node_id(&self) -> NodeId {
        self.node_id
    }
}

impl PartialEq for BasicAstInfo {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl SpannedAstInfo for BasicAstInfo {
    fn span(&self) -> &Span {
        &self.span
    }
}

pub trait BasicAstNode: AstNode<BasicAstInfo> {}

impl<T> BasicAstNode for T where T: AstNode<BasicAstInfo> {}

#[derive(Default)]
struct AstInfoGenerator {
    curr_id: u64,
}

impl GenerateAstInfo for AstInfoGenerator {
    type Result = BasicAstInfo;

    fn generate(&mut self, pair: &ParsePairSort) -> Self::Result {
        let res = BasicAstInfo {
            span: pair.span(),
            node_id: NodeId::new(self.curr_id),
        };
        self.curr_id += 1;
        res
    }
}

pub fn generate_ast<AST>(pairs: &ParsePairSort) -> AST
where
    AST: AstNode<BasicAstInfo>,
{
    AST::from_pairs(pairs, &mut AstInfoGenerator::default())
}

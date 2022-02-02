use crate::codegen_prelude::{AstInfo, GenerateAstInfo, ParsePairSort};
use crate::parser::ast::{AstNode, SpannedAstInfo};
use crate::sources::span::Span;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BasicAstInfo {
    span: Span,
}

impl AstInfo for BasicAstInfo {}
impl SpannedAstInfo for BasicAstInfo {
    fn span(&self) -> &Span {
        &self.span
    }
}

struct AstInfoGenerator;

impl GenerateAstInfo for AstInfoGenerator {
    type Result = BasicAstInfo;

    fn generate(&mut self, pair: &ParsePairSort) -> Self::Result {
        BasicAstInfo { span: pair.span() }
    }
}

pub fn generate_ast<AST>(pairs: &ParsePairSort) -> AST
where
    AST: AstNode<BasicAstInfo>,
{
    AST::from_pairs(pairs, &mut AstInfoGenerator)
}

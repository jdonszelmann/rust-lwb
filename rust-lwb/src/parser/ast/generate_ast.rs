use std::marker::PhantomData;
use crate::codegen_prelude::{AstInfo, GenerateAstInfo, ParsePairExpression, ParsePairSort};
use crate::parser::ast::{AstNode, SpannedAstInfo};
use crate::sources::span::Span;

pub struct BasicAstInfo<'src> {
    span: Span<'src>
}

impl<'src> AstInfo<'src> for BasicAstInfo<'src> {}
impl<'src> SpannedAstInfo<'src> for BasicAstInfo<'src> {
    fn span(&self) -> &Span<'src> {
        &self.span
    }
}

struct AstInfoGenerator<'src>(PhantomData<&'src ()>);
impl AstInfoGenerator<'_> {
    pub fn new() -> Self {
        Self(Default::default())
    }
}

impl<'src> GenerateAstInfo<'src> for AstInfoGenerator<'src> {
    type Result = BasicAstInfo<'src>;

    fn generate(&mut self, pair: &ParsePairSort<'src>) -> Self::Result {
        BasicAstInfo {
            span: pair.span()
        }
    }
}

pub fn generate_ast<'src, AST>(pairs: &ParsePairSort<'src>) -> AST
    where AST: AstNode<BasicAstInfo<'src>> {

    AST::from_pairs(pairs, &mut AstInfoGenerator::new())
}

use crate::parser::ast::AstInfo;
use crate::parser::peg::parser_pair::ParsePairSort;
use thiserror::Error;

pub trait GenerateAstInfo<'src> {
    type Result: AstInfo;

    fn generate(&mut self, pair: &ParsePairSort<'src>) -> Self::Result;
}

impl<'src, F, M: AstInfo> GenerateAstInfo<'src> for F
where
    F: FnMut(&ParsePairSort<'src>) -> M,
{
    type Result = M;

    fn generate(&mut self, pair: &ParsePairSort<'src>) -> Self::Result {
        (self)(pair)
    }
}

#[derive(Debug, Error)]
pub enum FromPairsError {}

pub trait FromPairs<'src, M: AstInfo<'src>> {
    fn from_pairs<G: GenerateAstInfo<'src, Result = M>>(pair: &ParsePairSort, generator: &mut G) -> Self
    where
        Self: Sized;
}

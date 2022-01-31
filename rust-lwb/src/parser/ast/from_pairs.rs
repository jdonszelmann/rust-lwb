use crate::parser::ast::AstInfo;
use crate::parser::peg::parser_pair::ParsePairSort;
use thiserror::Error;

pub trait GenerateAstInfo {
    type Result: AstInfo;

    fn generate(&mut self, pair: &ParsePairSort) -> Self::Result;
}

impl<F, M: AstInfo> GenerateAstInfo for F
where
    F: FnMut(&ParsePairSort) -> M,
{
    type Result = M;

    fn generate(&mut self, pair: &ParsePairSort) -> Self::Result {
        (self)(pair)
    }
}

#[derive(Debug, Error)]
pub enum FromPairsError {}

pub trait FromPairs<M: AstInfo> {
    fn from_pairs<G: GenerateAstInfo<Result = M>>(pair: ParsePairSort, generator: G) -> Self
    where
        Self: Sized;
}

use crate::sources::character_class::CharacterClass;
use crate::sources::span::Span;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum CoreExpression<'src> {
    Name(&'src str),
    Literal(&'src str),
    //TODO temp
    Sequence(Vec<CoreExpression<'src>>),
    Repeat {
        subexpr: Box<CoreExpression<'src>>,
        min: u64,
        max: Option<u64>,
    },
    CharacterClass(CharacterClass),
    Choice(Vec<CoreExpression<'src>>),
    FlagNoLayout(Box<CoreExpression<'src>>),
    FlagNoErrors(Box<CoreExpression<'src>>, String),
}

#[derive(Debug, Clone)]
pub struct CoreAst<'src> {
    pub sorts: HashMap<&'src str, CoreExpression<'src>>,
    pub starting_sort: &'src str,
    pub layout: CharacterClass,
}

#[derive(Debug, Clone)]
pub enum ParsePairRaw {
    Name(Span, Box<ParsePairRaw>),
    List(Span, Vec<ParsePairRaw>),
    Choice(Span, usize, Box<ParsePairRaw>),
    Empty(Span),
    Error(Span),
}

impl ParsePairRaw {
    /// What span does this parse pair occupy?
    pub fn span(&self) -> Span {
        match self {
            ParsePairRaw::Name(span, _) => span,
            ParsePairRaw::List(span, _) => span,
            ParsePairRaw::Choice(span, _, _) => span,
            ParsePairRaw::Empty(span) => span,
            ParsePairRaw::Error(span) => span,
        }
            .clone()
    }
}

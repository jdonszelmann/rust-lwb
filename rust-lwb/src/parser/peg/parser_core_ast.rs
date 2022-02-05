use crate::sources::character_class::CharacterClass;
use crate::sources::span::Span;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum CoreExpression {
    Name(String),
    Sequence(Vec<CoreExpression>),
    Repeat {
        c: Box<CoreExpression>,
        min: u64,
        max: Option<u64>,
    },
    CharacterClass(CharacterClass),
    Choice(Vec<CoreExpression>),
}

#[derive(Debug, Clone)]
pub struct CoreAst {
    pub sorts: HashMap<String, CoreExpression>,
    pub starting_sort: String,
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

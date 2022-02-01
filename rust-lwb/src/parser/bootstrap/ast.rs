use crate::sources::character_class::CharacterClass;
use derive_more::Display;
use enum_iterator::IntoEnumIterator;

#[derive(Debug, Clone)]
pub struct Constructor {
    pub name: String,
    pub constructor: Expression,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Sort(String),
    Literal(String),
    Sequence(Vec<Expression>),
    Repeat {
        c: Box<Expression>,
        min: u64,
        max: Option<u64>,
    },
    CharacterClass(CharacterClass),
    Choice(Vec<Expression>),

    Negative(Box<Expression>),
    Positive(Box<Expression>),
}

#[derive(Debug, Clone, IntoEnumIterator, Display, Eq, PartialEq)]
pub enum Annotation {
    #[display(fmt = "no-pretty-print")]
    NoPrettyPrint,
    #[display(fmt = "no-layout")]
    NoLayout,
    #[display(fmt = "injection")]
    Injection,
}

#[derive(Debug, Clone)]
pub struct Sort {
    pub name: String,
    pub constructors: Vec<Constructor>,
}

#[derive(Debug)]
pub struct SyntaxFileAst {
    pub sorts: Vec<Sort>,
    pub starting_sort: String,
    pub layout: CharacterClass,
}

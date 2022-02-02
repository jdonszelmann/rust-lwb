use std::str::FromStr;
use crate::sources::character_class::CharacterClass;
use derive_more::Display;
use enum_iterator::IntoEnumIterator;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constructor {
    pub name: String,
    pub expression: Expression,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, IntoEnumIterator, Display, Serialize, Deserialize, PartialEq, Eq)]
pub enum Annotation {
    #[display(fmt = "no-pretty-print")]
    NoPrettyPrint,

    /// Don't accept layout in this rule and any child rule
    #[display(fmt = "no-layout")]
    NoLayout,

    #[display(fmt = "injection")]
    Injection,

    /// represent this constructor as a single string in the final ast
    #[display(fmt = "single-string")]
    SingleString,
}

impl FromStr for Annotation {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for i in Annotation::into_enum_iter() {
            if s == i.to_string() {
                return Ok(i)
            }
        }
        Err(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sort {
    pub name: String,
    pub constructors: Vec<Constructor>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyntaxFileAst {
    pub sorts: Vec<Sort>,
    pub starting_sort: String,
    pub layout: CharacterClass,
}

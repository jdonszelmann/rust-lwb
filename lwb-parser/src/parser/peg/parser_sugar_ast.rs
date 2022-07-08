use crate::sources::character_class::CharacterClass;
use derive_more::Display;
use enum_iterator::IntoEnumIterator;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constructor {
    pub documentation: Option<String>,
    pub name: String,
    pub expression: Expression,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Expression {
    Sort(String),
    Literal(String),
    Sequence(Vec<Expression>),
    Repeat {
        e: Box<Expression>,
        min: u64,
        max: Option<u64>,
    },
    CharacterClass(CharacterClass),
    Choice(Vec<Expression>),
    Delimited {
        e: Box<Expression>,
        delim: Box<Expression>,
        min: u64,
        max: Option<u64>,
        trailing: bool,
    },

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

    /// this rule does not appear in the ast anywhere its used.
    #[display(fmt = "hidden")]
    Hidden,
}

impl FromStr for Annotation {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for i in Annotation::into_enum_iter() {
            if s == i.to_string() {
                return Ok(i);
            }
        }
        Err(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sort {
    pub documentation: Option<String>,
    pub name: String,
    pub constructors: Vec<Constructor>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyntaxFileAst {
    pub sorts: HashMap<String, Sort>,
    pub starting_sort: String,
}

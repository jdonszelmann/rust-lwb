use crate::sources::character_class::CharacterClass;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[derive(Debug, Clone, Display, Serialize, Deserialize, PartialEq, Eq)]
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

    /// This rule gives an error when parsed
    #[display(fmt = "error")]
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sort {
    pub documentation: Option<String>,
    pub name: String,
    pub constructors: Vec<Constructor>,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyntaxFileAst {
    pub sorts: HashMap<String, Sort>,
    pub starting_sort: String,
}

use crate::parser::syntax_file::character_class::CharacterClass;
use enum_iterator::IntoEnumIterator;
use derive_more::Display;

#[derive(Debug)]
pub enum Constructor {
    Identifier(String),
    Literal(String),
    Sequence(Vec<Constructor>),
    Repeat{
        c: Box<Constructor>,
        min: u64,
        max: Option<u64>
    },
    CharacterClass(CharacterClass),
    Choice(Vec<Constructor>),

    Negative(Box<Constructor>),
    Positive(Box<Constructor>),
}

#[derive(Debug, IntoEnumIterator, Display)]
pub enum Annotation {
    #[display(fmt="no-pretty-print")]
    NoPrettyPrint,
    #[display(fmt="no-layout")]
    NoLayout,
}

#[derive(Debug)]
pub struct Sort {
    pub name: String,
    pub constructors: Vec<Constructor>,
    pub annotations: Vec<Annotation>
}

#[derive(Debug)]
pub struct SyntaxFileAst {
    pub sorts: Vec<Sort>,
    pub starting_rule: String,
    pub layout: CharacterClass,
}


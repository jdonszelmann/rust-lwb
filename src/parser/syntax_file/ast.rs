use crate::parser::syntax_file::character_class::CharacterClass;
use derive_more::Display;
use enum_iterator::IntoEnumIterator;

#[derive(Debug)]
pub struct TopLevelConstructor {
    pub name: String,
    pub constructor: Constructor,
}

#[derive(Debug)]
pub enum Constructor {
    Sort(String),
    Literal(String),
    Sequence(Vec<Constructor>),
    Repeat {
        c: Box<Constructor>,
        min: u64,
        max: Option<u64>,
    },
    CharacterClass(CharacterClass),
    Choice(Vec<Constructor>),

    Negative(Box<Constructor>),
    Positive(Box<Constructor>),
}

#[derive(Debug, IntoEnumIterator, Display)]
pub enum Annotation {
    #[display(fmt = "no-pretty-print")]
    NoPrettyPrint,
    #[display(fmt = "no-layout")]
    NoLayout,
}

#[derive(Debug)]
pub struct Sort {
    pub name: String,
    pub constructors: Vec<TopLevelConstructor>,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug)]
pub struct SyntaxFileAst {
    pub sorts: Vec<Sort>,
    pub starting_sort: String,
    pub layout: CharacterClass,
}

use crate::parser::syntax_file::character_class::CharacterClass;

pub enum Constructor {
    Identifier(String),
    Literal(String),
    Sequence(Vec<Constructor>),
    Repeat{
        c: Box<Constructor>,
        min: usize,
        max: usize
    },
    CharacterClass(CharacterClass),
    Choice(Vec<Constructor>),

    Negative(Box<Constructor>),
    Positive(Box<Constructor>),
}

pub enum Annotation {
    NoPrettyPrint,
    NoLayout,
}

pub struct Sort {
    name: String,
    constructors: Vec<Constructor>,
    annotations: Vec<Annotation>
}

pub struct SyntaxFileAst {
    rules: Vec<Sort>,
    starting_rule: String,
    layout: Constructor,
}


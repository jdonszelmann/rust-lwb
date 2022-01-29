
struct CharacterClass {
    /* todo */
}

impl CharacterClass {
    pub fn contains(c: char) -> bool {
        todo!()
    }
}

enum Constructor {
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

enum Annotation {
    NoPrettyPrint,
    NoLayout,
}

struct Sort {
    name: String,
    constructors: Vec<Constructor>,
    annotations: Vec<Annotation>
}

struct SyntaxFile {
    rules: Vec<Sort>,
    starting_rule: String,
    layout: Constructor,
}


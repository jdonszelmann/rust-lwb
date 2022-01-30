use crate::span::Span;

pub struct ParsePairSort {
    pub sort: String,
    pub constructor_name: String,
    pub constructor_value: ParsePairConstructor,
}

impl ParsePairSort {
    pub fn span(&self) -> Span {
        self.constructor_value.span()
    }
}

pub enum ParsePairConstructor {
    Sort(Span, Box<ParsePairSort>),
    List(Span, Vec<ParsePairConstructor>),
    Choice(Span, usize, Box<ParsePairConstructor>),
    Text(Span),
    Empty(Span),
}

impl ParsePairConstructor {
    pub fn span(&self) -> Span {
        match self {
            ParsePairConstructor::Sort(span, _) => span,
            ParsePairConstructor::List(span, _) => span,
            ParsePairConstructor::Choice(span, _, _) => span,
            ParsePairConstructor::Text(span) => span,
            ParsePairConstructor::Empty(span) => span,
        }.clone()
    }
}

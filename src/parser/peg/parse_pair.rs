use crate::span::Span;

/// A parse pair is a way of representing an AST, without actually using any datatypes that depend on the language definition.
/// This represents a parse pair for a sort. It stores which constructor was chosen.
pub struct ParsePairSort {
    pub sort: String,
    pub constructor_name: String,
    pub constructor_value: ParsePairConstructor,
}

impl ParsePairSort {
    /// What span does this parse pair occupy?
    pub fn span(&self) -> Span {
        self.constructor_value.span()
    }
}

/// A parse pair is a way of representing an AST, without actually using any datatypes that depend on the language definition.
/// This represents a parse pair for a constructor. Each constructor generates one of the variants of this enum.
pub enum ParsePairConstructor {
    /// This is generated when another sort is mentioned in the definition of this sort.
    /// That sort is parsed and the result is stored here.
    Sort(Span, Box<ParsePairSort>),
    /// This is generated when a list of constructors is executed. This can be generated by Sequence or Repeat.
    List(Span, Vec<ParsePairConstructor>),
    /// This is generated when a Choice was made. The first argument is which choice was made, the second the parsed constructor.
    Choice(Span, usize, Box<ParsePairConstructor>),
    /// This is generated when no useful information needed to be recorded here, but still a placeholder is needed to keep track of the span.
    /// Generated by Positive and Negative, as the actual values that were parsed in Positive and Negative are irrelevant.
    /// Generated by CharacterClass and Literal, as they don't generate values.
    Empty(Span),
}

impl ParsePairConstructor {
    /// What span does this parse pair occupy?
    pub fn span(&self) -> Span {
        match self {
            ParsePairConstructor::Sort(span, _) => span,
            ParsePairConstructor::List(span, _) => span,
            ParsePairConstructor::Choice(span, _, _) => span,
            ParsePairConstructor::Empty(span) => span,
        }
        .clone()
    }
}

use std::cmp::Ordering;
use crate::parser::syntax_file::character_class::CharacterClass;
use crate::span::Span;
use thiserror::Error;

/// A parsing error represents a single error that occurred during parsing.
/// The parsing error occurs at a certain position in a file, represented by the span.
/// The parsing error consists of multiple `ParseErrorSub`, which each represent a single thing that went wrong at this position.
#[derive(Error, Debug, Clone)]
#[error("A parsing error occurred. Expected one of: {expected:?}")]
pub struct ParseError {
    pub span: Span,
    pub expected: Vec<ParseErrorSub>,
}

impl ParseError {
    pub fn expect_char_class(span: Span, val: CharacterClass) -> Self {
        ParseError { span, expected: vec![ParseErrorSub::ExpectCharClass(val)]}
    }

    pub fn expect_string(span: Span, val: String) -> Self {
        ParseError { span, expected: vec![ParseErrorSub::ExpectString(val)]}
    }

    pub fn not_entire_input(span: Span) -> Self {
        ParseError { span, expected: vec![ParseErrorSub::NotEntireInput()]}
    }
}


/// Represents a single thing that went wrong at this position.
#[derive(Debug, Clone)]
pub enum ParseErrorSub {
    /// Expect a character from a certain char class to be there, but it was not.
    ExpectCharClass(CharacterClass),

    /// Expect a certain string (keyword) to be there, but it was not.
    ExpectString(String),

    /// This happens when not the entire input was parsed, but also no errors occurred during parsing.
    NotEntireInput(),
}

impl ParseError {
    /// Combine multiple parse errors. When one has precedence over
    /// another, the highest precedence error is kept and the other
    /// is discarded.
    ///
    /// Highest precedence is defined as furthest starting position for now. This might be changed later.
    pub fn combine(mut self, mut other: ParseError) -> ParseError {
        assert_eq!(self.span.source.name(), other.span.source.name());

        //Compare the starting positions of the span
        match self.span.position.cmp(&other.span.position) {
            Ordering::Less => other,
            Ordering::Greater => self,
            Ordering::Equal => {
                //The span is extended such that the longest one is kept.
                self.span.length = self.span.length.max(other.span.length);
                //Merge the expected tokens
                self.expected.append(&mut other.expected);

                self
            }

        }
    }

    /// A helper that combines optional parse errors, and returns an optional parse error if either exists.
    /// If both exist, use `ParseError::combine` to combine the errors.
    pub fn combine_option_parse_error(a: Option<ParseError>, b: Option<ParseError>) -> Option<ParseError> {
        match (a, b) {
            (None, None) => None,
            (None, Some(e)) => Some(e),
            (Some(e), None) => Some(e),
            (Some(e1), Some(e2)) => Some(e1.combine(e2)),
        }
    }
}

use thiserror::Error;
use crate::parser::syntax_file::character_class::CharacterClass;
use crate::span::Span;

#[derive(Error, Debug, Clone)]
pub enum ParseError {

    /// Expect a character from a certain char class to be there, but it was not.
    #[error("expected char class: {1:?}")]
    ExpectCharClass(Span, CharacterClass),

    /// Expect a certain string to be there, but it was not.
    #[error("expected string: {1:?}")]
    ExpectString(Span, String),

    /// This happens when not the entire input was parsed, but also no errors occurred during parsing.
    #[error("not the entire input was parsed.")]
    NotEntireInput(Span),

    /// Combined errors represent a single position on which multiple errors occurred.
    /// They have strict rules, which should be enforced by the code constructing this variant:
    /// - All errors must occur in the same file.
    /// - All errors must be at the same `self.position()`
    #[error("multiple errors occurred: {0:?}")]
    CombinedError(Vec<ParseError>),
}

impl ParseError {
    /// Combine multiple parse errors. When one has precedence over
    /// another, the highest precedence error is kept and the other
    /// is discarded.
    ///
    /// When two errors are the same depth, they are merged into a single `CombinedError`.
    /// Otherwise, the error with the largest `self.position()` is chosen and the other is discarded.
    pub fn combine(self, other: ParseError) -> ParseError {
        todo!()
    }

    /// Parse errors always occur in a certain place. This returns at which span they occurred.
    pub fn span(&self) -> Span {
        todo!()
    }
}

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ParseError {


    /// Combined errors represent a single position on which multiple errors occurred.
    /// They have strict rules, which should be enforced by the code constructing this variant:
    /// - All errors must occur in the same file.
    /// - All errors must be at the same `self.position()`
    #[error("multiple errors occurred: {0:?}")]
    CombinedError(Vec<ParseError>)
}

impl ParseError {
    /// Combine multiple parse errors. When one has precedence over
    /// another, the highest precedence error is kept and the other
    /// is discarded.
    ///
    /// When two errors are the same depth, they are merged into a single `CombinedError`.
    /// Otherwise, the error with the largest `self.position()` is chosen and the other is discarded.
    pub fn combine(&self, _other: &ParseError) -> ParseError {
        todo!()
    }

    /// Parse errors always occur in a certain position. This returns at which position they occurred.
    pub fn position(&self) -> usize {
        todo!()
    }
}


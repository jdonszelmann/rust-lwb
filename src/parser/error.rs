use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ParseError {

    #[error("multiple errors occurred: {0:?}")]
    CombinedError(Vec<ParseError>)
}

impl ParseError {
    /// Combine multiple parse errors. When one has precedence over
    /// another, the highest precedence error is kept and the other
    /// is discarded. When two errors are the
    ///
    pub fn combine(&self, _other: &ParseError) -> ParseError {
        todo!()
    }
}


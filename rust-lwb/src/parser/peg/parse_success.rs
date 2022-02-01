use crate::sources::source_file::SourceFileIterator;

/// Represents a parser that parsed its value successfully.
/// It parsed the value of type `O`.
/// It also stores the best error encountered during parsing, and the position AFTER the parsed value in `pos`.
#[derive(Clone)]
pub struct ParseResult<'src, O: Clone> {
    pub result: O,
    pub pos: SourceFileIterator<'src>,
    pub success: bool,
}

impl<'src, O: Clone> ParseResult<'src, O> {
    /// Maps the result of this ParseSuccess, using a mapping function.
    pub fn map<F, ON: Clone>(self, mapfn: F) -> ParseResult<'src, ON>
    where
        F: Fn(O) -> ON,
    {
        ParseResult {
            result: mapfn(self.result),
            pos: self.pos,
            success: self.success,
        }
    }

    pub fn new_ok(result: O, pos: SourceFileIterator<'src>) -> Self {
        ParseResult {
            result,
            pos,
            success: true,
        }
    }

    pub fn new_err(result: O, pos: SourceFileIterator<'src>) -> Self {
        ParseResult {
            result,
            pos,
            success: false,
        }
    }
}

use crate::parser::peg::parse_error::ParseError;
use crate::sources::source_file::SourceFileIterator;

/// Represents a parser that parsed its value successfully.
/// It parsed the value of type `O`.
/// It also stores the best error encountered during parsing, and the position AFTER the parsed value in `pos`.
#[derive(Clone)]
pub struct ParseSuccess<'src, O: Clone> {
    pub result: O,
    pub best_error: Option<ParseError>,
    pub pos: SourceFileIterator<'src>,
}

impl<'a, O: Clone> ParseSuccess<'a, O> {
    /// Maps the result of this ParseSuccess, using a mapping function.
    pub fn map<F, ON: Clone>(self, mapfn: F) -> ParseSuccess<'a, ON>
    where
        F: Fn(O) -> ON,
    {
        ParseSuccess {
            result: mapfn(self.result),
            best_error: self.best_error,
            pos: self.pos,
        }
    }
}
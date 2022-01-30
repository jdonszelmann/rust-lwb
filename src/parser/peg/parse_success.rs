use crate::parser::peg::parse_error::{ParseError};
use crate::source_file::SourceFileIterator;

#[derive(Clone)]
pub struct ParseSuccess<'a, O> {
    pub result: O,
    pub best_error: Option<ParseError>,
    pub pos: SourceFileIterator<'a>,
}

impl<'a, O> ParseSuccess<'a, O> {
    pub fn map<F, ON>(self, mapfn: F) -> ParseSuccess<'a, ON> where F: Fn(O) -> ON {
        ParseSuccess { result: mapfn(self.result), best_error: self.best_error, pos: self.pos }
    }
}
use crate::parser::error::ParseError;
use crate::source_file::SourceFileIterator;

#[derive(Clone)]
pub struct ParseSuccess<'a, O> {
    pub result: O,
    pub best_error: Option<ParseError>,
    pub pos: SourceFileIterator<'a>,
}


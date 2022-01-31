use crate::sources::source_file::SourceFile;
use miette::{MietteError, SourceCode, SourceSpan, SpanContents};

/// Represents a certain range of a file. This is useful for marking the locations that certain tokens or errors occur.
/// The position and length are both in BYTES. The byte offsets provided should be valid.
#[derive(Clone, Debug)]
pub struct Span<'src> {
    pub position: usize,
    pub length: usize,
    pub source: &'src SourceFile,
}

impl<'src> Span<'src> {
    /// Creates a new span, given a file, starting position and the length that the span should be.
    pub fn from_length(source: &'src SourceFile, position: usize, length: usize) -> Self {
        Self {
            source,
            position,
            length,
        }
    }

    /// Creates a new span, given a file, starting position and end position.
    pub fn from_end(source: &'src SourceFile, position: usize, end: usize) -> Self {
        assert!(end >= position);
        Self {
            source,
            position,
            length: end - position,
        }
    }
}

impl SourceCode for Span<'_> {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
        <str as SourceCode>::read_span(
            self.source.contents(),
            span,
            context_lines_before,
            context_lines_after,
        )
    }
}

impl From<Span<'_>> for SourceSpan {
    fn from(span: Span) -> Self {
        SourceSpan::from((span.position, span.length))
    }
}

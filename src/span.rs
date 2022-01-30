use crate::source_file::SourceFile;

/// Represents a certain range of a file. This is useful for marking the locations that certain tokens or errors occur.
/// The position and length are both in BYTES. The byte offsets provided should be valid.
#[derive(Clone, Debug)]
pub struct Span {
    pub position: usize,
    pub length: usize,
    pub source: SourceFile,
}

impl Span {
    pub fn from_length(source: SourceFile, position: usize, length: usize) -> Self {
        Self {
            source, position, length
        }
    }

    pub fn from_end(source: SourceFile, position: usize, end: usize) -> Self {
        assert!(end >= position);
        Self {
            source, position, length: end - position
        }
    }
}
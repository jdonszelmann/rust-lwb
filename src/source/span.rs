use crate::source::source_file::SourceFile;

#[derive(Clone)]
pub struct Span {
    position: usize,
    length: usize,
    source: SourceFile,
}

// is_empty makes no sense for spans
#[allow(clippy::len_without_is_empty)]
impl Span {
    /// Returns the part of the source file described
    /// by this span.
    pub fn as_str(&self) -> &str {
        &self.source.contents()[self.start()..self.end()]
    }

    /// returns the location of the first
    /// character in this span
    pub fn start(&self) -> usize {
        self.position
    }

    /// returns the size of this span.
    pub fn len(&self) -> usize {
        self.length
    }

    /// Returns the location of the first character
    /// not in this span.
    pub fn end(&self) -> usize {
        self.position + self.length
    }
}

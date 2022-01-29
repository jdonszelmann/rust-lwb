use std::iter::Peekable;
use std::rc::Rc;

#[doc(hidden)]
struct Inner {
    contents: String,
    name: String
}

/// SourceFile represents a source into which spans
/// point. Source files can be cheaply cloned as the
/// actual contents of them live behind an `Rc`.
#[derive(Clone)]
pub struct SourceFile(Rc<Inner>);

impl SourceFile {
    /// Create a new SourceFile
    pub fn new(contents: String, name: String) -> Self {
        Self(Rc::new(Inner {
            contents,
            name
        }))
    }

    pub fn new_for_test(s: &str) -> Self {
        Self::new(s.to_string(), "test".to_string())
    }

    pub fn iter(&self) -> SourceFileIterator {
        SourceFileIterator {
            inner_iter: self.0.contents.chars().peekable()
        }
    }

    /// returns the name of this source file
    pub fn name(&self) -> &str {
        &self.0.name
    }

    /// returns the contents of this source file as a
    /// string. When parsing you likely often want to
    /// use `.iter()` instead as the source file iterator
    /// has a number of methods useful for parsing.
    pub fn contents(&self) -> &str {
        &self.0.contents
    }
}


pub struct SourceFileIterator<'a> {
    inner_iter: Peekable<std::str::Chars<'a>>,
}

impl<'a> SourceFileIterator<'a> {
    /// Peek at the next character that can be obtained
    /// by calling [`next`] or [`accept`].
    pub fn peek(&mut self) -> Option<&char> {
        self.inner_iter.peek()
    }

    /// Advance to the next character, discarding any
    /// character or error that is encountered.
    pub fn advance(&mut self) {
        let _ = self.inner_iter.next();
    }

    /// When the next value in the iterator is `c`, advance
    /// the iterator and return true. Otherwise, return false.
    pub fn accept(&mut self, c: char) -> bool {
        if self.peek() == Some(&c) {
            self.advance();
            true
        } else {
            false
        }
    }
}

impl<'a> Iterator for SourceFileIterator<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner_iter.next()
    }
}
use crate::sources::character_class::CharacterClass;
use miette::{MietteError, SourceCode, SourceSpan, SpanContents};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::io;
use std::io::Read;
use std::iter::Peekable;
use std::path::Path;
use std::sync::Arc;

#[doc(hidden)]
#[derive(Debug, Serialize, Deserialize)]
struct Inner {
    contents: String,
    name: String,
}

/// SourceFile represents a source into which spans
/// point. Source files can be cheaply cloned as the
/// actual contents of them live behind an `Rc`.
#[derive(Clone, Debug)]
pub struct SourceFile(Arc<Inner>);

impl Serialize for SourceFile {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_unit()
    }
}

impl<'de> Deserialize<'de> for SourceFile {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(SourceFile::new("", "dummmy"))
    }
}

impl SourceFile {
    pub fn open(name: impl AsRef<Path>) -> io::Result<Self> {
        let mut f = std::fs::File::open(&name)?;
        let mut contents = String::new();

        f.read_to_string(&mut contents)?;

        Ok(Self(Arc::new(Inner {
            contents,
            name: name.as_ref().to_string_lossy().to_string(),
        })))
    }

    /// Create a new SourceFile
    pub fn new(contents: impl AsRef<str>, name: impl AsRef<str>) -> Self {
        Self(Arc::new(Inner {
            contents: contents.as_ref().to_string(),
            name: name.as_ref().to_string(),
        }))
    }

    pub fn new_for_test(s: impl AsRef<str>) -> Self {
        Self::new(s.as_ref(), "test")
    }

    pub fn iter(&self) -> SourceFileIterator {
        SourceFileIterator {
            inner_iter: self.0.contents.chars().peekable(),
            index: 0,
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

#[derive(Clone)]
pub struct SourceFileIterator<'a> {
    inner_iter: Peekable<std::str::Chars<'a>>,
    index: usize,
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
        self.next();
    }

    /// Skip n characters.
    pub fn skip_n(&mut self, n: usize) {
        for _ in 0..n {
            self.advance();
        }
    }

    pub fn max_pos(&mut self, other: Self) {
        if other.index > self.index {
            *self = other;
        }
    }

    /// When the next value in the iterator is `c`, advance
    /// the iterator and return true. Otherwise, return false.
    ///
    /// ```
    /// # use rust_lwb::sources::source_file::SourceFile;
    /// let sf = SourceFile::new_for_test("test");
    /// let mut sfi = sf.iter();
    ///
    /// assert!(sfi.accept(&'t'.into()));
    ///
    /// // because the previous accept accepted
    /// // a 't' we will now see an e
    /// assert_eq!(sfi.peek(), Some(&'e'));
    /// assert_eq!(sfi.next(), Some('e'));
    /// sfi.advance();
    /// assert!(sfi.accept(&'t'.into()));
    ///
    /// // can't accept more, iterator is exhausted
    /// assert!(!sfi.accept(&'x'.into()));
    /// ```
    pub fn accept(&mut self, c: &CharacterClass) -> bool {
        self.accept_option(c).is_some()
    }

    /// Like accepts but returns an option
    pub fn accept_option(&mut self, c: &CharacterClass) -> Option<char> {
        if let Some(true) = self.peek().map(|&i| c.contains(i)) {
            self.next()
        } else {
            None
        }
    }

    /// accept an entire string. Returns true  only
    /// if the whole string could be accepted.
    ///
    /// ```
    /// # use rust_lwb::sources::source_file::SourceFile;
    /// let sf = SourceFile::new_for_test("test");
    /// let mut sfi = sf.iter();
    ///
    /// assert!(sfi.accept_str("test"));
    /// assert!(!sfi.accept_str("test"));
    /// assert!(sfi.exhausted());
    ///
    /// let mut sfi = sf.iter();
    /// assert!(sfi.accept_str("te"));
    /// assert!(sfi.accept_str("st"));
    /// assert!(sfi.exhausted());
    ///
    /// let mut sfi = sf.iter();
    /// assert!(!sfi.accept_str("cat"));
    /// assert!(sfi.accept_str("test"));
    /// assert!(sfi.exhausted());
    /// ```
    pub fn accept_str(&mut self, s: &str) -> bool {
        let mut self_clone = self.clone();
        for c in s.chars() {
            if !self_clone.accept(&c.into()) {
                return false;
            }
        }

        *self = self_clone;
        true
    }

    /// Skips any layout (defined by the layout character class passed in)
    ///
    /// ```
    /// # use rust_lwb::sources::source_file::SourceFile;
    /// let sf = SourceFile::new_for_test("   test");
    /// let mut sfi = sf.iter();
    ///
    /// assert!(!sfi.accept_str("test"));
    /// sfi.skip_layout(&' '.into());
    /// assert!(sfi.accept_str("test"));
    /// ```
    pub fn skip_layout(&mut self, layout: &CharacterClass) {
        while self.accept(layout) {}
    }

    /// First skip any layout that can be found, then accept like [`accept`]
    ///
    /// ```
    /// # use rust_lwb::sources::source_file::SourceFile;
    /// let sf = SourceFile::new_for_test("   t");
    /// let mut sfi = sf.iter();
    ///
    /// assert!(!sfi.accept(&'t'.into()));
    /// assert!(sfi.accept_skip_layout(&'t'.into(), &' '.into()));
    /// ```
    pub fn accept_skip_layout(&mut self, c: &CharacterClass, layout: &CharacterClass) -> bool {
        let mut self_clone = self.clone();
        self_clone.skip_layout(layout);
        if self_clone.accept(c) {
            *self = self_clone;
            true
        } else {
            false
        }
    }

    /// First skip any layout that can be found, then accept the string like [`accept_str`].
    ///
    /// ```
    /// # use rust_lwb::sources::source_file::SourceFile;
    /// let sf = SourceFile::new_for_test("   test");
    /// let mut sfi = sf.iter();
    ///
    /// assert!(!sfi.accept_str("test"));
    /// assert!(sfi.accept_str_skip_layout("test", &' '.into()));
    /// ```
    pub fn accept_str_skip_layout(&mut self, s: &str, layout: &CharacterClass) -> bool {
        let mut self_clone = self.clone();
        self_clone.skip_layout(layout);
        if self_clone.accept_str(s) {
            *self = self_clone;
            true
        } else {
            false
        }
    }

    /// accepts until a certain character is found in the input.
    ///
    /// ```
    /// # use rust_lwb::sources::source_file::SourceFile;
    /// let sf = SourceFile::new_for_test("test   ");
    /// let mut sfi = sf.iter();
    ///
    /// assert_eq!(sfi.accept_to_next(&' '.into()), "test");
    /// ```
    pub fn accept_to_next(&mut self, target: &CharacterClass) -> String {
        let mut res = String::new();

        while let Some(&i) = self.peek() {
            if target.contains(i) {
                break;
            } else {
                res.push(i);
                self.advance();
            }
        }

        res
    }

    /// Returns true if this iter won't return more
    /// ```
    /// # use rust_lwb::sources::source_file::SourceFile;
    /// let sf = SourceFile::new_for_test("test");
    /// let mut sfi = sf.iter();
    ///
    /// assert!(sfi.accept_str("test"));
    /// assert!(sfi.exhausted());
    pub fn exhausted(&mut self) -> bool {
        self.peek().is_none()
    }

    /// Returns the position of the character that is next.
    /// ```
    /// # use rust_lwb::sources::source_file::SourceFile;
    /// let sf = SourceFile::new_for_test("test");
    /// let mut sfi = sf.iter();
    ///
    /// assert_eq!(sfi.position(), 0);
    /// assert!(sfi.accept_str("tes"));
    /// assert_eq!(sfi.position(), 3);
    /// sfi.advance();
    /// assert_eq!(sfi.position(), 4);
    /// sfi.advance(); //Already at the end, so it has no effect on position
    /// assert_eq!(sfi.position(), 4);
    pub fn position(&self) -> usize {
        self.index
    }
}

impl<'a> Iterator for SourceFileIterator<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner_iter.next();
        if let Some(next) = next {
            self.index += next.len_utf8();
        }
        next
    }
}

impl SourceCode for SourceFile {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
        <str as SourceCode>::read_span(
            self.contents(),
            span,
            context_lines_before,
            context_lines_after,
        )
    }
}

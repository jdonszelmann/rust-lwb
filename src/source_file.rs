use crate::parser::syntax_file::character_class::CharacterClass;
use std::iter::{Enumerate, Peekable};
use std::rc::Rc;

#[doc(hidden)]
#[derive(Debug)]
struct Inner {
    contents: String,
    name: String,
}

/// SourceFile represents a source into which spans
/// point. Source files can be cheaply cloned as the
/// actual contents of them live behind an `Rc`.
#[derive(Clone, Debug)]
pub struct SourceFile(Rc<Inner>);

impl SourceFile {
    /// Create a new SourceFile
    pub fn new(contents: String, name: String) -> Self {
        Self(Rc::new(Inner { contents, name }))
    }

    pub fn new_for_test(s: &str) -> Self {
        Self::new(s.to_string(), "test".to_string())
    }

    pub fn iter(&self) -> SourceFileIterator {
        SourceFileIterator {
            inner_iter: self.0.contents.chars().peekable(),
            index: 0
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

    /// When the next value in the iterator is `c`, advance
    /// the iterator and return true. Otherwise, return false.
    ///
    /// ```
    /// # use rust_lwb::source_file::SourceFile;
    /// let sf = SourceFile::new_for_test("test");
    /// let mut sfi = sf.iter();
    ///
    /// assert!(sfi.accept('t'));
    ///
    /// // because the previous accept accepted
    /// // a 't' we will now see an e
    /// assert_eq!(sfi.peek(), Some(&'e'));
    /// assert_eq!(sfi.next(), Some('e'));
    /// sfi.advance();
    /// assert!(sfi.accept('t'));
    ///
    /// // can't accept more, iterator is exhausted
    /// assert!(!sfi.accept('x'));
    /// ```
    pub fn accept(&mut self, c: impl Into<CharacterClass>) -> bool {
        self.accept_option(c).is_some()
    }

    /// Like accepts but returns an option
    pub fn accept_option(&mut self, c: impl Into<CharacterClass>) -> Option<char> {
        let c = c.into();
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
    /// # use rust_lwb::source_file::SourceFile;
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
            if !self_clone.accept(c) {
                return false;
            }
        }

        *self = self_clone;
        true
    }

    /// Skips any layout (defined by the layout character class passed in)
    ///
    /// ```
    /// # use rust_lwb::source_file::SourceFile;
    /// let sf = SourceFile::new_for_test("   test");
    /// let mut sfi = sf.iter();
    ///
    /// assert!(!sfi.accept_str("test"));
    /// sfi.skip_layout(' ');
    /// assert!(sfi.accept_str("test"));
    /// ```
    pub fn skip_layout(&mut self, layout: impl Into<CharacterClass>) {
        let layout = layout.into();

        // TODO: have accept somehow accept references so this clone is not necessary
        while self.accept(layout.clone()) {}
    }

    /// First skip any layout that can be found, then accept like [`accept`]
    ///
    /// ```
    /// # use rust_lwb::source_file::SourceFile;
    /// let sf = SourceFile::new_for_test("   t");
    /// let mut sfi = sf.iter();
    ///
    /// assert!(!sfi.accept('t'));
    /// assert!(sfi.accept_skip_layout('t', ' '));
    /// ```
    pub fn accept_skip_layout(
        &mut self,
        c: impl Into<CharacterClass>,
        layout: impl Into<CharacterClass>,
    ) -> bool {
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
    /// # use rust_lwb::source_file::SourceFile;
    /// let sf = SourceFile::new_for_test("   test");
    /// let mut sfi = sf.iter();
    ///
    /// assert!(!sfi.accept_str("test"));
    /// assert!(sfi.accept_str_skip_layout("test", ' '));
    /// ```
    pub fn accept_str_skip_layout(&mut self, s: &str, layout: impl Into<CharacterClass>) -> bool {
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
    /// # use rust_lwb::source_file::SourceFile;
    /// let sf = SourceFile::new_for_test("test   ");
    /// let mut sfi = sf.iter();
    ///
    /// assert_eq!(sfi.accept_to_next(' '), "test");
    /// ```
    pub fn accept_to_next(&mut self, target: impl Into<CharacterClass>) -> String {
        let target = target.into();

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
    /// # use rust_lwb::source_file::SourceFile;
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
    /// # use rust_lwb::source_file::SourceFile;
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
    pub fn position(&mut self) -> usize {
        self.index
    }
}

impl<'a> Iterator for SourceFileIterator<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner_iter.next();
        if next.is_some() {
            self.index += next.unwrap().len_utf8();
        }
        next
    }
}

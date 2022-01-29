use std::ops::{Range, RangeInclusive};

/// Represent a class of characters like in a regex
/// such as [a-z] or [^0-9]
#[derive(Clone)]
pub enum CharacterClass {
    /// Inclusive range. Both `from` and `to` are inclusive
    RangeInclusive {
        from: char, // inclusive!
        to: char, // inclusive!
    },
    /// Exclusive range. `from` is inclusive but `to` is exclusive
    Range {
        from: char, // inclusive!
        to: char, // exclusive!
    },
    /// True when one of the character class parts is true
    Choice(Vec<CharacterClass>),
    /// inverts the outcome of the embedded character class
    Not(Box<CharacterClass>),
    /// Always false. Use Not(Nothing) for always true.
    Nothing,
}

impl CharacterClass {
    /// Contains returns true when a character is
    /// included in this character class.
    ///
    /// ```
    /// # use rust_lwb::parser::syntax_file::character_class::CharacterClass;
    ///
    /// let c = CharacterClass::from('a'..='z');
    /// assert!(c.contains('a'));
    /// assert!(c.contains('z'));
    /// assert!(!c.contains('0'));
    /// ```
    ///
    /// ```
    /// # use rust_lwb::parser::syntax_file::character_class::CharacterClass;
    ///
    /// // exclusive range so does not contain 'z'
    /// let c = CharacterClass::from('a'..'z');
    /// assert!(c.contains('a'));
    /// assert!(c.contains('y'));
    /// assert!(!c.contains('z'));
    /// assert!(!c.contains('0'));
    /// ```
    ///
    /// ```
    /// # use rust_lwb::parser::syntax_file::character_class::CharacterClass;
    ///
    /// // always return false
    /// let c = CharacterClass::Nothing;
    /// assert!(!c.contains('a'));
    /// assert!(!c.contains('0'));
    /// ```
    ///
    /// ```
    /// # use rust_lwb::parser::syntax_file::character_class::CharacterClass;
    ///
    /// // always return true
    /// let c = CharacterClass::Nothing.invert();
    /// assert!(c.contains('a'));
    /// assert!(c.contains('0'));
    /// ```
    pub fn contains(&self, c: char) -> bool {
        match self {
            CharacterClass::RangeInclusive { from, to } => {
                c >= *from && c <= *to
            }
            CharacterClass::Range { from, to } => {
                c >= *from && c < *to
            }
            CharacterClass::Choice(parts) => {
                parts.iter()
                    .map(|i| i.contains(c))
                    .fold(false, |i, j| i || j)
            }
            CharacterClass::Not(cls) => {
                !cls.contains(c)
            }
            CharacterClass::Nothing => false,
        }
    }


    pub fn invert(self) -> Self {
        Self::Not(Box::new(self))
    }

    /// Combine two character classes such that the result
    /// contains all characters from either of the two character
    /// class sets.
    ///
    /// ```
    /// use rust_lwb::parser::syntax_file::character_class::CharacterClass;
    ///
    /// let a = CharacterClass::from('a'..'z');
    /// let b = CharacterClass::from('0'..'9');
    /// assert!(a.contains('a'));
    /// assert!(!a.contains('0'));
    /// assert!(!b.contains('a'));
    /// assert!(b.contains('0'));
    ///
    /// let c = a.combine(b);
    /// assert!(c.contains('a'));
    /// assert!(c.contains('0'));
    /// ```
    pub fn combine(self, other: CharacterClass) -> CharacterClass {
        CharacterClass::Choice(vec![self, other])
    }
}

impl From<RangeInclusive<char>> for CharacterClass {
    fn from(r: RangeInclusive<char>) -> Self {
        Self::RangeInclusive {
            from: *r.start(),
            to: *r.end()
        }
    }
}

impl From<Range<char>> for CharacterClass {
    fn from(r: Range<char>) -> Self {
        Self::Range {
            from: r.start,
            to: r.end
        }
    }
}

impl From<char> for CharacterClass {
    fn from(c: char) -> Self {
        Self::Range {
            from: c,
            to: c
        }
    }
}

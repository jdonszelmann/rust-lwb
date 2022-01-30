use std::ops::{Range, RangeInclusive};

/// Represent a class of characters like in a regex
/// such as [a-z] or [^0-9]
#[derive(Clone, Debug)]
pub enum CharacterClass<'a> {
    /// Inclusive range. Both `from` and `to` are inclusive
    RangeInclusive {
        from: char,
        // inclusive!
        to: char,   // inclusive!
    },
    /// Exclusive range. `from` is inclusive but `to` is exclusive
    Range {
        from: char,
        // inclusive!
        to: char,   // exclusive!
    },
    /// all characters in the vec are in the character class.
    Contained(Vec<char>),
    /// True when one of the character class parts is true
    Choice(Vec<CharacterClass<'a>>),
    /// inverts the outcome of the embedded character class
    Not(Box<CharacterClass<'a>>),
    /// Always false. Use Not(Nothing) for always true.
    Nothing,

    /// Cow-like functionality for character classes.
    Ref(&'a CharacterClass<'a>),
}

impl<'a> CharacterClass<'a> {
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
                c as u32 >= *from as u32 && c as u32 <= *to as u32
            }
            CharacterClass::Range { from, to } => {
                (c as u32) >= *from as u32 && (c as u32) < *to as u32
            }
            CharacterClass::Choice(parts) => parts.iter().map(|i| i.contains(c)).any(|i| i),
            CharacterClass::Not(cls) => !cls.contains(c),
            CharacterClass::Nothing => false,
            CharacterClass::Contained(chars) => chars.contains(&c),
            CharacterClass::Ref(cls) => cls.contains(c),
        }
    }

    /// returns a character class that contains all elements
    /// of the slice.
    pub const fn all_in_vec(chars: Vec<char>) -> Self {
        Self::Contained(chars)
    }

    /// Invert this character class. The new class accepts any character
    /// not in the original character class
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
    pub fn combine(self, other: impl Into<CharacterClass<'a>>) -> CharacterClass<'a> {
        CharacterClass::Choice(vec![self, other.into()])
    }

    pub fn combine_ref(&'a self, other: impl Into<CharacterClass<'a>>) -> CharacterClass<'a> {
        CharacterClass::Ref(self).combine(other)
    }

    pub fn to_static(self) -> CharacterClass<'static> {
        match self {
            CharacterClass::Ref(cc) => {
                cc.clone().to_static()
            }
            CharacterClass::RangeInclusive { from, to } => CharacterClass::RangeInclusive {from, to},
            CharacterClass::Range { from, to } => CharacterClass::Range {from, to},
            CharacterClass::Contained(c) => CharacterClass::Contained(c),
            CharacterClass::Choice(c) => CharacterClass::Choice(c.into_iter().map(|i| i.to_static()).collect()),
            CharacterClass::Not(c) => CharacterClass::Not(Box::new(c.to_static())),
            CharacterClass::Nothing => CharacterClass::Nothing,
        }
    }
}

impl<'a> From<&'a CharacterClass<'a>> for CharacterClass<'a> {
    fn from(cc: &'a CharacterClass<'a>) -> Self {
        CharacterClass::Ref(cc)
    }
}

impl<const N: usize> From<[char; N]> for CharacterClass<'_> {
    fn from(chars: [char; N]) -> Self {
        chars.as_slice().into()
    }
}

impl From<RangeInclusive<char>> for CharacterClass<'_> {
    fn from(r: RangeInclusive<char>) -> Self {
        Self::RangeInclusive {
            from: *r.start(),
            to: *r.end(),
        }
    }
}

impl From<Range<char>> for CharacterClass<'_> {
    fn from(r: Range<char>) -> Self {
        Self::Range {
            from: r.start,
            to: r.end,
        }
    }
}

impl From<char> for CharacterClass<'_> {
    fn from(c: char) -> Self {
        Self::RangeInclusive { from: c, to: c }
    }
}

impl From<&[char]> for CharacterClass<'_> {
    fn from(s: &[char]) -> Self {
        Self::Contained(s.to_vec())
    }
}

impl From<Vec<char>> for CharacterClass<'_> {
    fn from(s: Vec<char>) -> Self {
        Self::Contained(s)
    }
}

impl From<String> for CharacterClass<'_> {
    fn from(s: String) -> Self {
        Self::Contained(s.chars().collect())
    }
}

impl<'a> From<&'a str> for CharacterClass<'_> {
    fn from(s: &'a str) -> Self {
        Self::Contained(s.chars().collect())
    }
}

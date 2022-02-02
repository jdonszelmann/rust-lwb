use crate::codegen_prelude::ParsePairSort;
use crate::parser::bootstrap::ast::{Sort};
use crate::parser::peg::parse_error::{ParseError};
use crate::parser::peg::parse_success::ParseSuccess;
use crate::sources::character_class::CharacterClass;
use crate::sources::source_file::SourceFile;
use std::collections::{HashMap, VecDeque};

/// This stores the immutable data that is used during the parsing process.
pub struct ParserState<'src> {
    pub(crate) file: &'src SourceFile,
    pub(crate) rules: HashMap<&'src str, &'src Sort>,
    pub layout: CharacterClass,
}

/// This stores the mutable data that is used during the parsing process.
/// It contains a cache of the results of each (source position, rule).
/// It also has a stack which contains information about the order in which the keys were inserted, so they can be removed in order when needed.
pub struct ParserCache<'src> {
    pub(crate) cache: HashMap<(usize, &'src str), ParserCacheEntry<'src>>,
    pub(crate) cache_stack: VecDeque<(usize, &'src str)>,
    pub best_error: Option<ParseError>,
    pub trace: VecDeque<&'src Sort>,
    pub allow_layout: bool, // True if layout should be allowed at the moment
    pub no_layout_nest_count: usize, // How many times no layout has been nested
    pub no_errors_nest_count: usize, // How many times no errors has been nested
}

/// A single entry in the cache. Contains the value, and a flag whether it has been read.
pub struct ParserCacheEntry<'src> {
    read: bool,
    value: Result<ParseSuccess<'src, ParsePairSort<'src>>, ()>,
}

#[derive(Copy, Clone)]
pub struct ParserFlags {}

impl<'src> ParserCache<'src> {
    /// Get a mutable reference to an entry
    pub fn get_mut(
        &mut self,
        key: &(usize, &'src str),
    ) -> Option<&mut Result<ParseSuccess<'src, ParsePairSort<'src>>, ()>> {
        if let Some(v) = self.cache.get_mut(key) {
            v.read = true;
            Some(&mut v.value)
        } else {
            None
        }
    }

    /// Check if an entry has been read
    pub fn is_read(&self, key: &(usize, &'src str)) -> Option<bool> {
        self.cache.get(key).map(|v| v.read)
    }

    /// Insert a new entry into the cache
    pub fn insert(
        &mut self,
        key: (usize, &'src str),
        value: Result<ParseSuccess<'src, ParsePairSort<'src>>, ()>,
    ) {
        self.cache
            .insert(key, ParserCacheEntry { read: false, value });
        self.cache_stack.push_back(key);
    }

    /// Check how many items are currently in the stack
    pub fn state_current(&self) -> usize {
        self.cache_stack.len()
    }

    /// Remove all the items that were inserted after the given stack marker
    pub fn state_revert(&mut self, state: usize) {
        self.cache_stack.drain(state..).for_each(|key| {
            self.cache.remove(&key);
        })
    }

    pub fn add_error(&mut self, error: ParseError) {
        if self.no_errors_nest_count > 0 {
            return;
        }
        match self.best_error.take() {
            Some(old_error) => self.best_error = Some(ParseError::combine(old_error, error)),
            None => self.best_error = Some(error),
        }
    }
}

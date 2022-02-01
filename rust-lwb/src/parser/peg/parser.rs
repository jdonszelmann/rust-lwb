use crate::codegen_prelude::ParsePairSort;
use crate::parser::bootstrap::ast::{Sort, SyntaxFileAst};
use crate::parser::peg::parse_error::{Expect, ParseError};
use crate::parser::peg::parser_sort::parse_sort;
use crate::sources::source_file::SourceFile;
use crate::sources::span::Span;
use std::collections::{HashMap, VecDeque};
use crate::parser::peg::parse_success::ParseResult;

/// This stores the immutable data that is used during the parsing process.
pub struct ParserInfo<'src> {
    pub(crate) file: &'src SourceFile,
    pub(crate) rules: HashMap<&'src str, &'src Sort>,
}

/// This stores the mutable data that is used during the parsing process.
/// It contains a cache of the results of each (source position, rule).
/// It also has a stack which contains information about the order in which the keys were inserted, so they can be removed in order when needed.
pub struct ParserState<'src> {
    cache: HashMap<(usize, &'src str), ParserCacheEntry<'src>>,
    cache_stack: VecDeque<(usize, &'src str)>,
    trace: VecDeque<&'src Sort>,
    best_error: Option<ParseError>,
}

/// A single entry in the cache. Contains the value, and a flag whether it has been read.
pub struct ParserCacheEntry<'src> {
    read: bool,
    value: ParseResult<'src, ParsePairSort<'src>>,
}

impl<'src> ParserState<'src> {
    /// Get a mutable reference to an entry
    pub fn get_mut(
        &mut self,
        key: &(usize, &'src str),
    ) -> Option<&mut ParseResult<'src, ParsePairSort<'src>>> {
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
        value: ParseResult<'src, ParsePairSort<'src>>,
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

    /// Start trace
    pub fn trace_start(&mut self, state: &'src Sort) {
        self.trace.push_back(state);
    }

    /// End trace
    pub fn trace_end(&mut self) {
        self.trace.pop_back().unwrap();
    }

    pub fn error(&mut self, error: ParseError) {
        match self.best_error.take() {
            Some(old_error) => self.best_error = Some(ParseError::combine(old_error, error)),
            None => self.best_error = Some(error),
        }
    }
}

/// Parses a file, given the syntax to parse it with, and the file.
/// When successful, it returns a `ParsePairSort`.
/// When unsuccessful, it returns a `ParseError`.
pub fn parse_file<'src>(
    syntax: &'src SyntaxFileAst,
    file: &'src SourceFile,
) -> Result<ParsePairSort<'src>, ParseError> {
    //Create a new parser state
    let mut state = ParserInfo {
        file,
        rules: HashMap::new(),
    };
    syntax.sorts.iter().for_each(|rule| {
        state.rules.insert(&rule.name, rule);
    });

    let mut cache = ParserState {
        cache: HashMap::new(),
        cache_stack: VecDeque::new(),
        trace: VecDeque::new(),
        best_error: None,
    };

    //Parse the starting sort
    let start = *state
        .rules
        .get(&syntax.starting_sort[..])
        .expect("Starting sort exists");
    let mut res = parse_sort(&state, &mut cache, start, file.iter());
    if !res.success {
        return Err(cache.best_error.unwrap());
    }

    //If there is no input left, return Ok.
    if res.pos.peek().is_none() {
        Ok(res.result)
    } else {
        //If any occurred during the parsing, return it. Otherwise, return a generic NotEntireInput error.
        //I'm not entirely sure this logic always returns relevant errors. Maybe we should inform the user the parse was actually fine, but didn't parse enough?
        match cache.best_error {
            Some(err) => Err(err),
            None => {
                let curpos = res.pos.position();
                while res.pos.next().is_some() {}
                let endpos = res.pos.position();
                Err(ParseError::expect(
                    Span::from_end(file, curpos, endpos),
                    Expect::NotEntireInput(),
                ))
            }
        }
    }
}

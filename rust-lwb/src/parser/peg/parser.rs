use crate::codegen_prelude::ParsePairSort;
use crate::parser::bootstrap::ast::{Sort, SyntaxFileAst};
use crate::parser::peg::parse_error::PEGParseError;
use crate::parser::peg::parse_success::ParseSuccess;
use crate::parser::peg::parser_sort::parse_sort;
use crate::sources::character_class::CharacterClass;
use crate::sources::source_file::SourceFile;
use crate::sources::span::Span;
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
    cache: HashMap<(usize, &'src str), ParserCacheEntry<'src>>,
    cache_stack: VecDeque<(usize, &'src str)>,
    pub trace: VecDeque<&'src Sort>,
    pub allow_layout: bool, // True if layout should be allowed at the moment
    pub no_layout_nest_count: usize, // How many times no layout has been nested
}

#[derive(Copy, Clone)]
pub struct ParserFlags {
}

impl<'src> ParserCache<'src> {
    /// Get a mutable reference to an entry
    pub fn get_mut(
        &mut self,
        key: &(usize, &'src str),
    ) -> Option<&mut Result<ParseSuccess<'src, ParsePairSort<'src>>, PEGParseError>> {
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
        value: Result<ParseSuccess<'src, ParsePairSort<'src>>, PEGParseError>,
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
}

/// A single entry in the cache. Contains the value, and a flag whether it has been read.
pub struct ParserCacheEntry<'src> {
    read: bool,
    value: Result<ParseSuccess<'src, ParsePairSort<'src>>, PEGParseError>,
}

/// Parses a file, given the syntax to parse it with, and the file.
/// When successful, it returns a `ParsePairSort`.
/// When unsuccessful, it returns a `ParseError`.
pub fn parse_file<'src>(
    syntax: &'src SyntaxFileAst, // TODO: are these lifetimes truly the same?
    file: &'src SourceFile,      // TODO: the same as this one I mean
) -> Result<ParsePairSort<'src>, PEGParseError> {
    //Create a new parser state
    let mut state = ParserState {
        file,
        rules: HashMap::new(),
        layout: syntax.layout.clone(),
    };
    syntax.sorts.iter().for_each(|rule| {
        state.rules.insert(&rule.name, rule);
    });

    let mut cache = ParserCache {
        cache: HashMap::new(),
        cache_stack: VecDeque::new(),
        trace: VecDeque::new(),
        no_layout_nest_count: 0usize,
        allow_layout: true,
    };

    let flags = ParserFlags {

    };

    //Parse the starting sort
    let starting_sort = state
        .rules
        .get(&syntax.starting_sort[..])
        .expect("Starting sort exists");
    let mut ok: ParseSuccess<ParsePairSort<'src>> =
        parse_sort(&state, &mut cache, flags, starting_sort, file.iter())?;

    //If there is no input left, return Ok.
    if ok.pos.peek().is_none() {
        Ok(ok.result)
    } else {
        //If any occurred during the parsing, return it. Otherwise, return a generic NotEntireInput error.
        //I'm not entirely sure this logic always returns relevant errors. Maybe we should inform the user the parse was actually fine, but didn't parse enough?
        match ok.best_error {
            Some(err) => Err(err),
            None => {
                let curpos = ok.pos.position();
                while ok.pos.next().is_some() {}
                let endpos = ok.pos.position();
                Err(PEGParseError::not_entire_input(Span::from_end(
                    file, curpos, endpos,
                )))
            }
        }
    }
}

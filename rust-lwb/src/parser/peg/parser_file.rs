use crate::codegen_prelude::ParsePairSort;
use crate::parser::bootstrap::ast::{Sort, SyntaxFileAst};
use crate::parser::peg::parse_error::{Expect, PEGParseError};
use crate::parser::peg::parser::{ParserState, ParserContext};
use crate::parser::peg::parser_sort::parse_sort;
use crate::sources::source_file::{SourceFile, SourceFileIterator};
use crate::sources::span::Span;
use std::collections::{HashMap, VecDeque};
use crate::parser::peg::parse_result::ParseResult;

/// Parses a file, given the syntax to parse it with, and the file.
/// When successful, it returns a `ParsePairSort`.
/// When unsuccessful, it returns a `ParseError`.
pub fn parse_file<'src>(
    syntax: &'src SyntaxFileAst,
    file: &'src SourceFile,
) -> (ParsePairSort<'src>, Vec<PEGParseError>) {
    //Create a new parser state
    let mut state = ParserContext {
        file,
        rules: HashMap::new(),
        layout: syntax.layout.clone(),
        errors: Vec::new(),
    };
    syntax.sorts.iter().for_each(|rule| {
        state.rules.insert(&rule.name, rule);
    });

    //Parse the starting sort
    let starting_sort = state
        .rules
        .get(&syntax.starting_sort[..])
        .expect("Starting sort exists");

    let mut errors = vec![];
    let last_pos = 0usize;
    loop {
        let (res, err) = parse_file_sub(&state, starting_sort, file.iter());
        if !res.ok {
            let err = err.expect("Not ok means an error happened.");
            state.errors.push(res.pos_err.position());
            errors.push(err);
        } else {
            return (res.result, errors)
        }
    }
}

pub fn parse_file_sub<'src>(
    state: &ParserContext<'src>,
    sort: &'src Sort,
    pos: SourceFileIterator<'src>,
) -> (ParseResult<'src, ParsePairSort<'src>>, Option<PEGParseError>) {
    let mut cache = ParserState {
        cache: HashMap::new(),
        cache_stack: VecDeque::new(),
        best_error: None,
        trace: VecDeque::new(),
        no_layout_nest_count: 0usize,
        no_errors_nest_count: 0usize,
        allow_layout: true,
    };

    let mut res = parse_sort(&state, &mut cache, sort, pos);
    if !res.ok {
        return (res, Some(cache.best_error.unwrap()));
    }

    //If there is no input left, return Ok.
    res.pos.skip_layout(&state.layout);

    if res.pos.peek().is_none() {
        (res, None)
    } else {
        //If any occurred during the parsing, return it. Otherwise, return a generic NotEntireInput error.
        //I'm not entirely sure this logic always returns relevant errors. Maybe we should inform the user the parse was actually fine, but didn't parse enough?
        res.ok = false;
        match cache.best_error {
            Some(err) => {
                (res, Some(err))
            },
            None => {
                let curpos = res.pos.position();
                while res.pos.next().is_some() {}
                let endpos = res.pos.position();
                (res, Some(PEGParseError::expect(
                    Span::from_end(state.file, curpos, endpos),
                    Expect::NotEntireInput(),
                )))
            }
        }
    }
}

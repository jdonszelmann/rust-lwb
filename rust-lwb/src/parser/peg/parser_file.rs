use crate::codegen_prelude::ParsePairSort;
use crate::parser::bootstrap::ast::{Sort, SyntaxFileAst};
use crate::parser::peg::parse_error::{Expect, PEGParseError};
use crate::parser::peg::parse_result::ParseResult;
use crate::parser::peg::parser::{ParserContext, ParserState};
use crate::parser::peg::parser_sort::parse_sort;
use crate::sources::source_file::{SourceFile, SourceFileIterator};
use crate::sources::span::Span;
use std::collections::{HashMap, VecDeque};

/// Parses a file, given the syntax to parse it with, and the file.
/// When successful, it returns a `ParsePairSort`.
/// When unsuccessful, it returns a `ParseError`.
#[allow(clippy::unnecessary_unwrap)] //Clippy gives a suggestion which makes code ugly
pub fn parse_file<'src>(
    syntax: &'src SyntaxFileAst,
    file: &'src SourceFile,
) -> (ParsePairSort<'src>, Vec<PEGParseError>) {
    //Create a new parser state
    let mut state = ParserContext {
        file,
        rules: HashMap::new(),
        layout: syntax.layout.clone(),
        errors: HashMap::new(),
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

    let mut last_err_pos: Option<usize> = None;
    let mut last_err_offset = 0usize;
    loop {
        let (res, err) = parse_file_sub(&state, starting_sort, file.iter());
        if !res.ok {
            let err = err.expect("Not ok means an error happened.");

            //If this is the first time we encounter this, error, log it and retry
            if last_err_pos.is_none()
                || last_err_pos.unwrap() + last_err_offset < res.pos_err.position()
            {
                errors.push(err);
                last_err_pos = Some(res.pos_err.position());
                last_err_offset = 0;
                state.errors.insert(last_err_pos.unwrap(), last_err_offset);

                continue;
            } else {
                //If the error now spans rest of file, we could not recover
                let len_left = res.pos_err.clone().count();
                if last_err_offset >= len_left {
                    return (res.result, errors);
                }

                //Increase offset by 1 and repeat
                last_err_offset += 1;
                state.errors.insert(last_err_pos.unwrap(), last_err_offset);
            }
        } else {
            return (res.result, errors);
        }
    }
}

pub fn parse_file_sub<'src>(
    state: &ParserContext<'src>,
    sort: &'src Sort,
    pos: SourceFileIterator<'src>,
) -> (
    ParseResult<'src, ParsePairSort<'src>>,
    Option<PEGParseError>,
) {
    let mut cache = ParserState {
        cache: HashMap::new(),
        cache_stack: VecDeque::new(),
        best_error: None,
        trace: VecDeque::new(),
        no_layout_nest_count: 0usize,
        no_errors_nest_count: 0usize,
        allow_layout: true,
    };

    let mut res = parse_sort(state, &mut cache, sort, pos);
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
            Some(err) => (res, Some(err)),
            None => {
                let curpos = res.pos.position();
                while res.pos.next().is_some() {}
                let endpos = res.pos.position();
                (
                    res,
                    Some(PEGParseError::expect(
                        Span::from_end(state.file, curpos, endpos),
                        &cache.trace,
                        Expect::NotEntireInput(),
                    )),
                )
            }
        }
    }
}

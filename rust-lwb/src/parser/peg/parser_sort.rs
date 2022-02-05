#![allow(clippy::result_unit_err)]

use crate::codegen_prelude::{ParsePairExpression, ParsePairSort};
use crate::parser::bootstrap::ast::{Annotation, Sort};
use crate::parser::peg::parse_error::{Expect, PEGParseError};
use crate::parser::peg::parse_result::ParseResult;
use crate::parser::peg::parser::{ParserContext, ParserState};
use crate::parser::peg::parser_expression::parse_expression;
use crate::sources::source_file::SourceFileIterator;
use crate::sources::span::Span;

/// Given the name of a sort and the current position, attempts to parse this sort.
/// The name of the provided sort must exist.
pub fn parse_sort<'src>(
    state: &ParserContext<'src>,
    cache: &mut ParserState<'src>,
    sort: &'src Sort,
    pos: SourceFileIterator<'src>,
) -> ParseResult<'src, ParsePairSort<'src>> {
    //Check if this result is cached
    let key = (pos.position(), &sort.name[..]);
    if let Some(cached) = cache.get_mut(&key) {
        return cached.clone();
    }

    //Before executing, put a value for the current position in the cache.
    //This value is used if the rule is left-recursive
    let cache_state = cache.state_current();
    cache.insert(
        key,
        ParseResult::new_err(
            ParsePairSort {
                sort: &sort.name,
                constructor_name: "ERROR",
                constructor_value: ParsePairExpression::Error(Span::from_length(
                    state.file,
                    pos.position(),
                    0,
                )),
            },
            pos.clone(),
            pos.clone(),
        ),
    );

    //Now execute the actual rule, taking into account left recursion
    //The way this is done is heavily inspired by http://web.cs.ucla.edu/~todd/research/pepm08.pdf
    //A quick summary
    //- First put an error value for the current (rule, position) in the cache
    //- Try to parse the current (rule, position). If this fails, there is definitely no left recursion. Otherwise, we now have a seed.
    //- Put the new seed in the cache, and rerun on the current (rule, position). Make sure to revert the cache to the previous state.
    //- At some point, the above will fail. Either because no new input is parsed, or because the entire parse now failed. At this point, we have reached the maximum size.
    let mut res = parse_sort_sub(state, cache, sort, pos.clone());
    let res = if res.ok {
        //Do we have a leftrec case?
        if !cache.is_read(&key).unwrap() {
            //There was no leftrec, just return the value
            res
        } else {
            //There was leftrec, we need to grow the seed
            loop {
                //Insert the current seed into the cache
                cache.state_revert(cache_state);
                cache.insert(key, res.clone());

                //Grow the seed
                let new_res = parse_sort_sub(state, cache, sort, pos.clone());
                if !new_res.ok {
                    break;
                }
                if new_res.pos.position() <= res.pos.position() {
                    break;
                }
                res = new_res;
            }
            //The seed is at its maximum size
            cache.insert(key, res.clone());
            res
        }
    } else {
        // Left recursion value was used, but did not make a seed.
        // This is an illegal grammar!
        if cache.is_read(&key).unwrap() {
            cache.add_error(PEGParseError::fail_left_recursion(Span::from_length(
                state.file,
                pos.position(),
                0,
            )));
        }
        res
    };

    cache.insert(key, res.clone());

    //Return result
    res
}
fn parse_sort_sub<'src>(
    state: &ParserContext<'src>,
    cache: &mut ParserState<'src>,
    sort: &'src Sort,
    pos: SourceFileIterator<'src>,
) -> ParseResult<'src, ParsePairSort<'src>> {
    //Try each constructor, keeping track of the best error that occurred while doing so.
    //If none of the constructors succeed, we will return this error.
    let mut results = vec![];
    assert!(!sort.constructors.is_empty());
    for constructor in &sort.constructors {
        cache.trace.push_back((sort, constructor));
        if constructor.annotations.contains(&Annotation::NoLayout) {
            cache.no_layout_nest_count += 1;
            cache.no_errors_nest_count += 1;
        }
        let res = parse_expression(state, cache, &constructor.expression, pos.clone());
        cache.trace.pop_back();
        if constructor.annotations.contains(&Annotation::NoLayout) {
            cache.no_layout_nest_count -= 1;
            cache.no_errors_nest_count -= 1;
            if cache.no_layout_nest_count == 0 {
                cache.allow_layout = true;
            }
        }

        if res.ok && !res.recovered {
            return ParseResult::new_ok(
                ParsePairSort {
                    sort: &sort.name,
                    constructor_name: &constructor.name,
                    constructor_value: res.result,
                },
                res.pos,
                res.pos_err,
                res.recovered,
            );
        }
        if constructor.annotations.contains(&Annotation::NoLayout) {
            let span = Span::from_length(state.file, pos.position(), 1);
            let err = PEGParseError::expect(
                span,
                &cache.trace,
                Expect::ExpectSort(sort.name.clone(), constructor.name.clone()),
            );
            cache.add_error(err);
        }
        results.push(res);
    }
    //Chose best candidate
    let (i, res) = results
        .into_iter()
        .enumerate()
        .max_by_key(|(_, r)| r.pos_err.position())
        .unwrap();
    if res.ok {
        ParseResult::new_ok(
            ParsePairSort {
                sort: &sort.name,
                constructor_name: &sort.constructors[i].name,
                constructor_value: res.result,
            },
            res.pos,
            res.pos_err,
            res.recovered
        )
    } else {
        ParseResult::new_err(
            ParsePairSort {
                sort: &sort.name,
                constructor_name: &sort.constructors[i].name,
                constructor_value: res.result,
            },
            res.pos,
            res.pos_err,
        )
    }

}

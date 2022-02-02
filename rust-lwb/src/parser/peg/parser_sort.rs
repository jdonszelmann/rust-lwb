use crate::codegen_prelude::ParsePairSort;
use crate::parser::bootstrap::ast::{Annotation, Sort};
use crate::parser::peg::parse_error::{Expect, ParseError};
use crate::parser::peg::parse_success::ParseSuccess;
use crate::parser::peg::parser::{ParserCache, ParserFlags, ParserState};
use crate::parser::peg::parser_expression::parse_expression;
use crate::sources::source_file::SourceFileIterator;
use crate::sources::span::Span;

/// Given the name of a sort and the current position, attempts to parse this sort.
/// The name of the provided sort must exist.
pub fn parse_sort<'src>(
    state: &ParserState<'src>,
    cache: &mut ParserCache<'src>,
    flags: ParserFlags,
    sort: &'src Sort,
    pos: SourceFileIterator<'src>,
) -> Result<ParseSuccess<'src, ParsePairSort<'src>>, ()> {
    //Check if this result is cached
    let key = (pos.position(), &sort.name[..]);
    if let Some(cached) = cache.get_mut(&key) {
        return cached.clone();
    }

    //Before executing, put a value for the current position in the cache.
    //This value is used if the rule is left-recursive
    let cache_state = cache.state_current();
    cache.insert(key, Err(()));
    cache.trace.push_back(sort);

    //Now execute the actual rule, taking into account left recursion
    //The way this is done is heavily inspired by http://web.cs.ucla.edu/~todd/research/pepm08.pdf
    //A quick summary
    //- First put an error value for the current (rule, position) in the cache
    //- Try to parse the current (rule, position). If this fails, there is definitely no left recursion. Otherwise, we now have a seed.
    //- Put the new seed in the cache, and rerun on the current (rule, position). Make sure to revert the cache to the previous state.
    //- At some point, the above will fail. Either because no new input is parsed, or because the entire parse now failed. At this point, we have reached the maximum size.
    let res = match parse_sort_sub(state, cache, flags, sort, pos.clone()) {
        Ok(mut ok) => {
            //Do we have a leftrec case?
            if !cache.is_read(&key).unwrap() {
                //There was no leftrec, just return the value
                Ok(ok)
            } else {
                //There was leftrec, we need to grow the seed
                loop {
                    //Insert the current seed into the cache
                    cache.state_revert(cache_state);
                    cache.insert(key, Ok(ok.clone()));

                    //Grow the seed
                    match parse_sort_sub(state, cache, flags, sort, pos.clone()) {
                        Ok(new_ok) => {
                            if new_ok.pos.position() <= ok.pos.position() {
                                break;
                            }
                            ok = new_ok;
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
                //The seed is at its maximum size
                cache.insert(key, Ok(ok.clone()));
                Ok(ok)
            }
        }
        //If it failed, we know
        Err(_) => {
            // Left recursion value was used, but did not make a seed.
            // This is an illegal grammar!
            if cache.is_read(&key).unwrap() {
                cache.add_error(ParseError::fail_left_recursion(Span::from_length(
                    state.file,
                    pos.position(),
                    0,
                )));
            }
            Err(())
        }
    };
    cache.insert(key, res.clone());

    cache.trace.pop_back();

    //Return result
    res
}
fn parse_sort_sub<'src>(
    state: &ParserState<'src>,
    cache: &mut ParserCache<'src>,
    flags: ParserFlags,
    sort: &'src Sort,
    pos: SourceFileIterator<'src>,
) -> Result<ParseSuccess<'src, ParsePairSort<'src>>, ()> {
    //Try each constructor, keeping track of the best error that occurred while doing so.
    //If none of the constructors succeed, we will return this error.
    assert!(sort.constructors.len() > 0);
    for constructor in &sort.constructors {
        if constructor.annotations.contains(&Annotation::NoLayout) {
            cache.no_layout_nest_count += 1;
            cache.no_errors_nest_count += 1;
        }
        let res = parse_expression(state, cache, flags, &constructor.constructor, pos.clone());
        if constructor.annotations.contains(&Annotation::NoLayout) {
            cache.no_layout_nest_count -= 1;
            cache.no_errors_nest_count -= 1;
            if cache.no_layout_nest_count == 0 {
                cache.allow_layout = true;
            }
        }

        match res {
            Ok(ok) => {
                return Ok(ParseSuccess {
                    result: ParsePairSort {
                        sort: &sort.name,
                        constructor_name: &constructor.name,
                        constructor_value: ok.result,
                    },
                    pos: ok.pos,
                });
            }
            Err(_) => {
                if constructor.annotations.contains(&Annotation::NoLayout) {
                    let span = Span::from_length(&state.file, pos.position(), 1);
                    let err = ParseError::expect(
                        span,
                        Expect::ExpectSort(sort.name.clone(), constructor.name.clone()),
                    );
                    cache.add_error(err);
                }
            }
        }
    }
    Err(())
}

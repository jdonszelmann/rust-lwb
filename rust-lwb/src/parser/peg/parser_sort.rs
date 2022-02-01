use crate::codegen_prelude::{ParsePairExpression, ParsePairSort};
use crate::parser::bootstrap::ast::{Expression, Sort};
use crate::parser::peg::parse_error::ParseError;
use crate::parser::peg::parse_success::ParseSuccess;
use crate::parser::peg::parser::{ParserInfo, ParserState};
use crate::parser::peg::parser_expression::parse_expression;
use crate::sources::source_file::SourceFileIterator;
use crate::sources::span::Span;

/// Given the name of a sort and the current position, attempts to parse this sort.
/// The name of the provided sort must exist.
pub fn parse_sort<'src>(
    state: &ParserInfo<'src>,
    cache: &mut ParserState<'src>,
    sort: &'src Sort,
    pos: SourceFileIterator<'src>,
) -> Result<ParseSuccess<'src, ParsePairSort<'src>>, ParseError> {
    //Check if this result is cached
    let key = (pos.position(), &sort.name[..]);
    if let Some(cached) = cache.get_mut(&key) {
        return cached.clone();
    }

    //Before executing, put a value for the current position in the cache.
    //This value is used if the rule is left-recursive
    cache.trace_start(sort);
    let cache_state = cache.state_current();
    cache.insert(
        key,
        Err(ParseError::fail_left_recursion(Span::from_length(
            state.file,
            pos.position(),
            0,
        ))),
    );

    //Now execute the actual rule, taking into account left recursion
    //The way this is done is heavily inspired by http://web.cs.ucla.edu/~todd/research/pepm08.pdf
    //A quick summary
    //- First put an error value for the current (rule, position) in the cache
    //- Try to parse the current (rule, position). If this fails, there is definitely no left recursion. Otherwise, we now have a seed.
    //- Put the new seed in the cache, and rerun on the current (rule, position). Make sure to revert the cache to the previous state.
    //- At some point, the above will fail. Either because no new input is parsed, or because the entire parse now failed. At this point, we have reached the maximum size.
    let res = match parse_sort_sub(state, cache, sort, pos.clone()) {
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
                    match parse_sort_sub(state, cache, sort, pos.clone()) {
                        Ok(new_ok) => {
                            if new_ok.pos.position() <= ok.pos.position() {
                                ok.best_error = ParseError::combine_option_parse_error(
                                    ok.best_error,
                                    new_ok.best_error,
                                );
                                break;
                            }
                            ok = new_ok;
                        }
                        Err(new_err) => {
                            ok.best_error = ParseError::combine_option_parse_error(
                                ok.best_error,
                                Some(new_err),
                            );
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
        Err(err) => Err(err),
    };
    cache.insert(key, res.clone());
    cache.trace_end();

    //Return result
    res
}
fn parse_sort_sub<'src>(
    state: &ParserInfo<'src>,
    cache: &mut ParserState<'src>,
    sort: &'src Sort,
    pos: SourceFileIterator<'src>,
) -> Result<ParseSuccess<'src, ParsePairSort<'src>>, ParseError> {
    //We need to make an ordered choice between the constructors
    //To do this, create a choice expression and parse that
    let expr = Expression::Choice(
        sort.constructors
            .iter()
            .map(|c| c.constructor.clone())
            .collect(),
    );
    match parse_expression(state, cache, &expr, pos.clone()) {
        Ok(ok) => {
            Ok(ok.map(|res| {
                //Map the ParsePairExpression to a ParsePairSort
                if let ParsePairExpression::Choice(_, cnum, cval) = res {
                    ParsePairSort {
                        sort: &sort.name,
                        constructor_name: &sort.constructors[cnum].name,
                        constructor_value: *cval,
                    }
                } else {
                    unreachable!()
                }
            }))
        }
        Err(err) => Err(err),
    }
}

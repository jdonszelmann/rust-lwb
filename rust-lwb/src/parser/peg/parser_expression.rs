use crate::codegen_prelude::{ParsePairExpression, ParsePairSort};
use crate::parser::bootstrap::ast::{Expression, Sort};
use crate::parser::peg::parse_error::{Expect, ParseError};
use crate::parser::peg::parse_success::ParseResult;
use crate::parser::peg::parser::{ParserInfo, ParserState};
use crate::parser::peg::parser_sort::parse_sort;
use crate::sources::source_file::SourceFileIterator;
use crate::sources::span::Span;

/// Given an expression and the current position, attempts to parse this constructor.
pub fn parse_expression<'src>(
    state: &ParserInfo<'src>,
    cache: &mut ParserState<'src>,
    constructor: &Expression,
    mut pos: SourceFileIterator<'src>,
) -> ParseResult<'src, ParsePairExpression<'src>> {
    match constructor {
        //To parse a sort, call parse_sort recursively.
        Expression::Sort(rule) => {
            let sort: &'src Sort = *state.rules.get(&rule[..]).expect("Sort exists");
            let res = parse_sort(state, cache, sort, pos);
            ParseResult {
                result: ParsePairExpression::Sort(res.result.span(), Box::new(res.result)),
                pos: res.pos,
                success: res.success
            }
        }
        //To parse a literal, use accept_str to check if it parses.
        Expression::Literal(lit) => {
            let span = Span::from_length(state.file, pos.position(), lit.len());
            if pos.accept_str(lit) {
                ParseResult::new_ok(ParsePairExpression::Empty(span), pos)
            } else {
                cache.error(ParseError::expect(
                    span.clone(),
                    Expect::ExpectString(lit.clone()),
                ));
                ParseResult::new_err(ParsePairExpression::Error(span), pos)
            }
        }
        //To parse a sequence, parse each constructor in the sequence.
        //The results are added to `results`, and the best error and position are updated each time.
        //Finally, construct a `ParsePairConstructor::List` with the results.
        Expression::Sequence(constructors) => {
            let mut results = vec![];
            let start_pos = pos.position();

            //Parse all subconstructors in sequence
            for subconstructor in constructors {
                let res = parse_expression(state, cache, subconstructor, pos);
                results.push(res.result);
                pos = res.pos;
                if !res.success {
                    let span = Span::from_end(state.file, start_pos, pos.position());
                    return ParseResult::new_err(ParsePairExpression::List(span, results), pos);
                }
            }

            //Construct result
            let span = Span::from_end(state.file, start_pos, pos.position());
            ParseResult::new_ok(ParsePairExpression::List(span, results), pos)
        }
        //To parse a sequence, first parse the minimum amount that is needed.
        //Then keep trying to parse the constructor until the maximum is reached.
        //The results are added to `results`, and the best error and position are updated each time.
        //Finally, construct a `ParsePairConstructor::List` with the results.
        Expression::Repeat { c, min, max } => {
            let mut results = vec![];
            let start_pos = pos.position();
            let mut last_pos = pos.position();

            //Parse minimum amount that is needed
            for _ in 0..*min {
                let res = parse_expression(state, cache, c.as_ref(), pos);
                results.push(res.result);
                pos = res.pos;
                if !res.success {
                    let span = Span::from_end(state.file, start_pos, pos.position());
                    return ParseResult::new_err(ParsePairExpression::List(span, results), pos);
                }

                //If the position hasn't changed, then we're in an infinite loop
                if last_pos == pos.position() {
                    let span = Span::from_length(state.file, pos.position(), 0);
                    cache.error(ParseError::fail_loop(span.clone()));
                    return ParseResult::new_err(ParsePairExpression::List(span, results), pos);
                }
                last_pos = pos.position();
            }

            //Parse until maximum amount is reached
            for _ in *min..max.unwrap_or(u64::MAX) {
                let res = parse_expression(state, cache, c.as_ref(), pos);
                results.push(res.result);
                pos = res.pos;
                if !res.success {
                    break;
                }

                //If the position hasn't changed, then we're in an infinite loop
                if last_pos == pos.position() {
                    let span = Span::from_length(state.file, pos.position(), 0);
                    cache.error(ParseError::fail_loop(span.clone()));
                    return ParseResult::new_err(ParsePairExpression::List(span, results), pos);
                }
                last_pos = pos.position();
            }

            //Construct result
            let span = Span::from_end(state.file, start_pos, pos.position());
            ParseResult::new_ok(ParsePairExpression::List(span, results), pos)
        }
        //To parse a character class, check if the character is accepted, and make an ok/error based on that.
        Expression::CharacterClass(characters) => {
            let span = Span::from_length(state.file, pos.position(), 1);
            if pos.accept(characters) {
                ParseResult::new_ok(ParsePairExpression::Empty(span), pos)
            } else {
                cache.error(ParseError::expect(
                    span.clone(),
                    Expect::ExpectCharClass(characters.clone()),
                ));
                ParseResult::new_err(ParsePairExpression::Error(span), pos)
            }
        }
        //To parse a choice, try each constructor, keeping track of the best error that occurred while doing so.
        //If none of the constructors succeed, we will return this error.
        Expression::Choice(constructors) => {
            let mut best_res_i = usize::MAX;
            let mut best_res: Option<ParseResult<ParsePairExpression>> = None;
            for (i, subconstructor) in constructors.iter().enumerate() {
                let res = parse_expression(state, cache, subconstructor, pos.clone());
                if res.success {
                    return ParseResult::new_ok(ParsePairExpression::Choice(res.result.span(), i, Box::new(res.result)), res.pos)
                } else {
                    if best_res.is_none() || res.pos.position() > best_res.as_ref().unwrap().pos.position() {
                        best_res_i = i;
                        best_res = Some(res);
                    }
                }
            }

            let res = best_res.unwrap();
            ParseResult::new_err(ParsePairExpression::Choice(res.result.span(), best_res_i, Box::new(res.result)), res.pos)
        }
        //To parse a negative, try parsing the constructor.
        //If it succeeds, we need to make an error, not sure how
        //If it fails, we return ok.
        Expression::Negative(_) => {
            todo!()
            // match parse_expression(state, cache, constructor.as_ref(), pos.clone()) {
            //     Ok(_) => {
            //         todo!() //Negatives are complicated with errors
            //     }
            //     Err(err) => {
            //         Ok(ParseSuccess {
            //             result: ParsePairExpression::Empty(err.span),
            //             best_error: None,
            //             pos, //Return old position
            //         })
            //     }
            // }
        }
        //To parse a positive, try parsing the constructor.
        //If it succeeds, we return ok. Otherwise, we return the error.
        Expression::Positive(_) => {
            todo!()
            // match parse_expression(state, cache, constructor.as_ref(), pos.clone()) {
            //     Ok(ok) => {
            //         Ok(ParseSuccess {
            //             result: ParsePairExpression::Empty(ok.result.span()),
            //             best_error: None, //If the positive passed, then we don't care about any "better" parses inside it
            //             pos,              //Return old position
            //         })
            //     }
            //     Err(err) => Err(err),
            // }
        }
    }
}

use crate::codegen_prelude::{ParsePairExpression, ParsePairSort};
use crate::parser::bootstrap::ast::Expression;
use crate::parser::peg::parse_error::ParseError;
use crate::parser::peg::parse_success::ParseSuccess;
use crate::parser::peg::parser::{ParserCache, ParserFlags, ParserState};
use crate::parser::peg::parser_sort::parse_sort;
use crate::sources::source_file::SourceFileIterator;
use crate::sources::span::Span;

/// Given an expression and the current position, attempts to parse this constructor.
pub fn parse_expression<'src>(
    state: &ParserState<'src>,
    cache: &mut ParserCache<'src>,
    constructor: &'src Expression,
    mut pos: SourceFileIterator<'src>,
    flags: ParserFlags
) -> Result<ParseSuccess<'src, ParsePairExpression<'src>>, ParseError> {
    match constructor {
        //To parse a sort, call parse_sort recursively.
        Expression::Sort(rule) => Ok(parse_sort(state, cache, rule, pos, flags)?
            .map(|s: ParsePairSort<'src>| ParsePairExpression::Sort(s.span(), Box::new(s)))),
        //To parse a literal, use accept_str to check if it parses.
        Expression::Literal(lit) => {
            //First, skip layout
            if !flags.no_layout { pos.skip_layout(&state.layout) }

            let span = Span::from_length(state.file, pos.position(), lit.len());
            if pos.accept_str(lit) {
                Ok(ParseSuccess {
                    result: ParsePairExpression::Empty(span),
                    best_error: None,
                    pos,
                })
            } else {
                Err(ParseError::expect_string(span, lit.clone()))
            }
        }
        //To parse a sequence, parse each constructor in the sequence.
        //The results are added to `results`, and the best error and position are updated each time.
        //Finally, construct a `ParsePairConstructor::List` with the results.
        Expression::Sequence(constructors) => {
            let mut results = vec![];
            let mut best_error = None;
            let start_pos = pos.position();

            //Parse all subconstructors in sequence
            for subconstructor in constructors {
                match parse_expression(state, cache, subconstructor, pos, flags) {
                    Ok(ok) => {
                        pos = ok.pos;
                        best_error =
                            ParseError::combine_option_parse_error(best_error, ok.best_error);
                        results.push(ok.result);
                    }
                    Err(err) => {
                        best_error = ParseError::combine_option_parse_error(best_error, Some(err));
                        return Err(best_error.unwrap());
                    }
                }
            }

            //Construct result
            let span = Span::from_end(state.file, start_pos, pos.position());
            Ok(ParseSuccess {
                result: ParsePairExpression::List(span, results),
                best_error,
                pos,
            })
        }
        //To parse a sequence, first parse the minimum amount that is needed.
        //Then keep trying to parse the constructor until the maximum is reached.
        //The results are added to `results`, and the best error and position are updated each time.
        //Finally, construct a `ParsePairConstructor::List` with the results.
        Expression::Repeat { c, min, max } => {
            let mut results = vec![];
            let mut best_error = None;
            let start_pos = pos.position();
            let mut last_pos = pos.position();

            //Parse minimum amount that is needed
            for _ in 0..*min {
                match parse_expression(state, cache, c.as_ref(), pos, flags) {
                    Ok(ok) => {
                        results.push(ok.result);
                        pos = ok.pos;
                        best_error =
                            ParseError::combine_option_parse_error(best_error, ok.best_error);
                    }
                    Err(err) => {
                        best_error = ParseError::combine_option_parse_error(best_error, Some(err));
                        return Err(best_error.unwrap());
                    }
                }
                //If the position hasn't changed, then we're in an infinite loop
                if last_pos == pos.position() {
                    let span = Span::from_length(state.file, pos.position(), 0);
                    // best_error = ParseError::combine_option_parse_error(best_error, Some(ParseError::fail_loop(span)));
                    return Err(ParseError::fail_loop(span));
                }
                last_pos = pos.position();
            }

            //Parse until maximum amount is reached
            for _ in *min..max.unwrap_or(u64::MAX) {
                match parse_expression(state, cache, c.as_ref(), pos.clone(), flags) {
                    Ok(ok) => {
                        results.push(ok.result);
                        pos = ok.pos;
                        best_error =
                            ParseError::combine_option_parse_error(best_error, ok.best_error);
                    }
                    Err(err) => {
                        best_error = ParseError::combine_option_parse_error(best_error, Some(err));
                        break;
                    }
                }
                //If the position hasn't changed, then we're in an infinite loop
                if last_pos == pos.position() {
                    let span = Span::from_length(state.file, pos.position(), 0);
                    // best_error = ParseError::combine_option_parse_error(best_error, Some(ParseError::fail_loop(span)));
                    return Err(ParseError::fail_loop(span));
                }
                last_pos = pos.position();
            }

            //Construct result
            let span = Span::from_end(state.file, start_pos, pos.position());
            Ok(ParseSuccess {
                result: ParsePairExpression::List(span, results),
                best_error,
                pos,
            })
        }
        //To parse a character class, check if the character is accepted, and make an ok/error based on that.
        Expression::CharacterClass(characters) => {
            //First, skip layout
            if !flags.no_layout { pos.skip_layout(&state.layout) }

            let span = Span::from_length(state.file, pos.position(), 1);
            if pos.accept(characters) {
                Ok(ParseSuccess {
                    result: ParsePairExpression::Empty(span),
                    best_error: None,
                    pos,
                })
            } else {
                Err(ParseError::expect_char_class(span, characters.clone()))
            }
        }
        //To parse a choice, try each constructor, keeping track of the best error that occurred while doing so.
        //If none of the constructors succeed, we will return this error.
        Expression::Choice(constructors) => {
            let mut best_error = None;
            for (i, subconstructor) in constructors.iter().enumerate() {
                match parse_expression(state, cache, subconstructor, pos.clone(), flags) {
                    Ok(suc) => {
                        best_error =
                            ParseError::combine_option_parse_error(best_error, suc.best_error);
                        return Ok(ParseSuccess {
                            result: ParsePairExpression::Choice(
                                suc.result.span(),
                                i,
                                Box::new(suc.result),
                            ),
                            pos: suc.pos,
                            best_error,
                        });
                    }
                    Err(err) => {
                        best_error = ParseError::combine_option_parse_error(best_error, Some(err))
                    }
                }
            }
            Err(best_error.unwrap())
        }
        //To parse a negative, try parsing the constructor.
        //If it succeeds, we need to make an error, not sure how
        //If it fails, we return ok.
        Expression::Negative(constructor) => {
            match parse_expression(state, cache, constructor.as_ref(), pos.clone(), flags) {
                Ok(_) => {
                    todo!() //Negatives are complicated with errors
                }
                Err(err) => {
                    Ok(ParseSuccess {
                        result: ParsePairExpression::Empty(err.span),
                        best_error: None,
                        pos, //Return old position
                    })
                }
            }
        }
        //To parse a positive, try parsing the constructor.
        //If it succeeds, we return ok. Otherwise, we return the error.
        Expression::Positive(constructor) => {
            match parse_expression(state, cache, constructor.as_ref(), pos.clone(), flags) {
                Ok(ok) => {
                    Ok(ParseSuccess {
                        result: ParsePairExpression::Empty(ok.result.span()),
                        best_error: None, //If the positive passed, then we don't care about any "better" parses inside it
                        pos,              //Return old position
                    })
                }
                Err(err) => Err(err),
            }
        }
    }
}

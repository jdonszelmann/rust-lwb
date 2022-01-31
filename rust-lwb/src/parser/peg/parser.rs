use crate::sources::source_file::SourceFile;
use crate::sources::source_file::SourceFileIterator;
use crate::sources::span::Span;
use std::collections::HashMap;
use crate::codegen_prelude::{ParsePairExpression, ParsePairSort};
use crate::parser::bootstrap::ast::{Expression, Sort, SyntaxFileAst};
use crate::parser::peg::parser_error::ParseError;
use crate::parser::peg::parser_success::ParseSuccess;

/// This stores the immutable data that is used during the parsing process.
struct ParserState<'src> {
    file: &'src SourceFile,
    rules: HashMap<&'src str, &'src Sort>,
}

/// This stores the mutable data that is used during the parsing process.
struct ParserCache<'src> {
    cache: HashMap<
        (usize, &'src str),
        Result<ParseSuccess<'src, ParsePairSort<'src>>, ParseError<'src>>,
    >,
}

/// Parses a file, given the syntax to parse it with, and the file.
/// When successful, it returns a `ParsePairSort`.
/// When unsuccessful, it returns a `ParseError`.
pub fn parse_file<'src>(
    syntax: &'src SyntaxFileAst,
    file: &'src SourceFile,
) -> Result<ParsePairSort<'src>, ParseError<'src>> {
    //Create a new parser state
    let mut state = ParserState {
        file,
        rules: HashMap::new(),
    };
    syntax.sorts.iter().for_each(|rule| {
        state.rules.insert(&rule.name, &rule);
    });

    let mut cache = ParserCache {
        cache: HashMap::new(),
    };

    //Parse the starting sort
    let mut ok: ParseSuccess<ParsePairSort<'src>> =
        parse_sort(&state, &mut cache, &syntax.starting_sort, file.iter())?;

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
                Err(ParseError::not_entire_input(Span::from_end(
                    &file, curpos, endpos,
                )))
            }
        }
    }
}

/// Given the name of a sort and the current position, attempts to parse this sort.
/// The name of the provided sort must exist.
fn parse_sort<'src>(
    state: &ParserState<'src>,
    cache: &mut ParserCache<'src>,
    sort: &'src str,
    pos: SourceFileIterator<'src>,
) -> Result<ParseSuccess<'src, ParsePairSort<'src>>, ParseError<'src>> {
    //Check if this result is cached
    let key = (pos.position(), sort);
    if let Some(cached) = cache.cache.get(&key) {
        return (*cached).clone();
    }

    //Before executing, put a value for the current position in the cache.
    //This value is used if the rule is left-recursive
    cache.cache.insert(
        key,
        Err(ParseError::left_recursion(Span::from_length(
            &state.file,
            pos.position(),
            0,
        ))),
    );

    //Now execute the actual rule
    let res = parse_sort_sub(state, cache, sort, pos);

    //Now update the cache with the real value
    *cache.cache.get_mut(&key).unwrap() = res.clone();

    //Return result
    res
}
fn parse_sort_sub<'src>(
    state: &ParserState<'src>,
    cache: &mut ParserCache<'src>,
    sort: &'src str,
    pos: SourceFileIterator<'src>,
) -> Result<ParseSuccess<'src, ParsePairSort<'src>>, ParseError<'src>> {
    //Obtain the sort
    let sort: &'src Sort = state.rules.get(sort).expect("Name is guaranteed to exist");

    //Try each constructor, keeping track of the best error that occurred while doing so.
    //If none of the constructors succeed, we will return this error.
    let mut best_error: Option<ParseError> = None;
    for constructor in &sort.constructors {
        match parse_constructor(state, cache, &constructor.constructor, pos.clone()) {
            Ok(ok) => {
                return Ok(ParseSuccess {
                    //TODO should be a bit smarter and avoid these clones
                    result: ParsePairSort {
                        sort: &sort.name,
                        constructor_name: &constructor.name,
                        constructor_value: ok.result,
                    },
                    //If one of the previous constructors had a better error, we should return that one
                    best_error: ok.best_error.or(best_error),
                    pos: ok.pos,
                });
            }
            Err(err) => best_error = ParseError::combine_option_parse_error(best_error, Some(err)),
        }
    }
    Err(best_error.expect("Each sort has at least one constructor"))
}

/// Given a constructor and the current position, attempts to parse this constructor.
fn parse_constructor<'src>(
    state: &ParserState<'src>,
    cache: &mut ParserCache<'src>,
    constructor: &'src Expression,
    mut pos: SourceFileIterator<'src>,
) -> Result<ParseSuccess<'src, ParsePairExpression<'src>>, ParseError<'src>> {
    match constructor {
        //To parse a sort, call parse_sort recursively.
        Expression::Sort(rule) => Ok(parse_sort(state, cache, rule, pos)?
            .map(|s: ParsePairSort<'src>| ParsePairExpression::Sort(s.span(), Box::new(s)))),
        //To parse a literal, use accept_str to check if it parses.
        Expression::Literal(lit) => {
            let span = Span::from_length(&state.file, pos.position(), lit.len());
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
                match parse_constructor(state, cache, subconstructor, pos) {
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
            let span = Span::from_end(&state.file, start_pos, pos.position());
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

            //Parse minimum amount that is needed
            for _ in 0..*min {
                match parse_constructor(state, cache, c.as_ref(), pos) {
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
            }

            //Parse until maximum amount is reached
            for _ in *min..max.unwrap_or(u64::MAX) {
                match parse_constructor(state, cache, c.as_ref(), pos.clone()) {
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
            }

            //Construct result
            let span = Span::from_end(&state.file, start_pos, pos.position());
            Ok(ParseSuccess {
                result: ParsePairExpression::List(span, results),
                best_error,
                pos,
            })
        }
        //To parse a character class, check if the character is accepted, and make an ok/error based on that.
        Expression::CharacterClass(characters) => {
            let span = Span::from_length(&state.file, pos.position(), 1);
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
                match parse_constructor(state, cache, subconstructor, pos.clone()) {
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
            match parse_constructor(state, cache, constructor.as_ref(), pos.clone()) {
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
            match parse_constructor(state, cache, constructor.as_ref(), pos.clone()) {
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

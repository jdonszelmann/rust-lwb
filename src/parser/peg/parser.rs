use crate::parser::peg::parse_error::ParseError;
use crate::parser::peg::parse_pair::{ParsePairConstructor, ParsePairSort};
use crate::parser::peg::parse_success::ParseSuccess;
use crate::parser::syntax_file::ast::{Constructor, Sort, SyntaxFileAst};
use crate::source_file::{SourceFile, SourceFileIterator};
use crate::span::Span;
use std::collections::HashMap;

/// This stores the data that is used during the parsing process.
struct ParserState {
    file: SourceFile,
    rules: HashMap<String, Sort>,
}

/// Parses a file, given the syntax to parse it with, and the file.
/// When successful, it returns a `ParsePairSort`.
/// When unsuccessful, it returns a `ParseError`.
pub fn parse_file(syntax: SyntaxFileAst, file: SourceFile) -> Result<ParsePairSort, ParseError> {
    //Create a new parser state
    let mut state = ParserState {
        file: file.clone(),
        rules: HashMap::new(),
    };
    syntax.sorts.into_iter().for_each(|rule| {
        state.rules.insert(rule.name.clone(), rule).unwrap();
    });

    //Parse the starting sort
    let mut ok = state.parse_sort(&syntax.starting_sort, file.iter())?;

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
                    file.clone(),
                    curpos,
                    endpos,
                )))
            }
        }
    }
}

impl ParserState {
    /// Given the name of a sort and the current position, attempts to parse this sort.
    /// The name of the provided sort must exist.
    fn parse_sort<'a>(
        &self,
        sort: &str,
        pos: SourceFileIterator<'a>,
    ) -> Result<ParseSuccess<'a, ParsePairSort>, ParseError> {
        //Obtain the sort
        let sort = self.rules.get(sort).unwrap(); //Safe: Name is guaranteed to exist.

        //Try each constructor, keeping track of the best error that occurred while doing so.
        //If none of the constructors succeed, we will return this error.
        let mut best_error: Option<ParseError> = None;
        for constructor in &sort.constructors {
            match self.parse_constructor(&constructor.constructor, pos.clone()) {
                Ok(ok) => {
                    return Ok(ParseSuccess {
                        //TODO should be a bit smarter and avoid these clones
                        result: ParsePairSort {
                            sort: sort.name.clone(),
                            constructor_name: constructor.name.clone(),
                            constructor_value: ok.result,
                        },
                        //If one of the previous constructors had a better error, we should return that one
                        best_error: ok.best_error.or(best_error),
                        pos,
                    });
                }
                Err(err) => {
                    best_error = ParseError::combine_option_parse_error(best_error, Some(err))
                }
            }
        }
        Err(best_error.unwrap()) //Safe: Each sort has at least one constructor
    }

    /// Given a constructor and the current position, attempts to parse this constructor.
    fn parse_constructor<'a>(
        &self,
        constructor: &Constructor,
        mut pos: SourceFileIterator<'a>,
    ) -> Result<ParseSuccess<'a, ParsePairConstructor>, ParseError> {
        match constructor {
            //To parse a sort, call parse_sort recursively.
            Constructor::Sort(rule) => Ok(self
                .parse_sort(rule, pos)?
                .map(|s| ParsePairConstructor::Sort(s.span(), Box::new(s)))),
            //To parse a literal, use accept_str to check if it parses.
            Constructor::Literal(lit) => {
                let span = Span::from_length(self.file.clone(), pos.position(), lit.len());
                if pos.accept_str(lit) {
                    Ok(ParseSuccess {
                        result: ParsePairConstructor::Text(span),
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
            Constructor::Sequence(constructors) => {
                let mut results = vec![];
                let mut best_error = None;
                let start_pos = pos.position();

                //Parse all subconstructors in sequence
                for subconstructor in constructors {
                    match self.parse_constructor(subconstructor, pos) {
                        Ok(ok) => {
                            pos = ok.pos;
                            best_error =
                                ParseError::combine_option_parse_error(best_error, ok.best_error);
                            results.push(ok.result);
                        }
                        Err(err) => {
                            best_error =
                                ParseError::combine_option_parse_error(best_error, Some(err));
                            return Err(best_error.unwrap());
                        }
                    }
                }

                //Construct result
                let span = Span::from_end(self.file.clone(), start_pos, pos.position());
                Ok(ParseSuccess {
                    result: ParsePairConstructor::List(span, results),
                    best_error,
                    pos,
                })
            }
            //To parse a sequence, first parse the minimum amount that is needed.
            //Then keep trying to parse the constructor until the maximum is reached.
            //The results are added to `results`, and the best error and position are updated each time.
            //Finally, construct a `ParsePairConstructor::List` with the results.
            Constructor::Repeat { c, min, max } => {
                let mut results = vec![];
                let mut best_error = None;
                let start_pos = pos.position();

                //Parse minimum amount that is needed
                for _ in 0..*min {
                    match self.parse_constructor(c.as_ref(), pos) {
                        Ok(ok) => {
                            results.push(ok.result);
                            pos = ok.pos;
                            best_error =
                                ParseError::combine_option_parse_error(best_error, ok.best_error);
                        }
                        Err(err) => {
                            best_error =
                                ParseError::combine_option_parse_error(best_error, Some(err));
                            return Err(best_error.unwrap());
                        }
                    }
                }

                //Parse until maximum amount is reached
                for _ in *min..max.unwrap_or(u64::MAX) {
                    match self.parse_constructor(c.as_ref(), pos.clone()) {
                        Ok(ok) => {
                            results.push(ok.result);
                            pos = ok.pos;
                            best_error =
                                ParseError::combine_option_parse_error(best_error, ok.best_error);
                        }
                        Err(err) => {
                            best_error =
                                ParseError::combine_option_parse_error(best_error, Some(err));
                            break;
                        }
                    }
                }

                //Construct result
                let span = Span::from_end(self.file.clone(), start_pos, pos.position());
                Ok(ParseSuccess {
                    result: ParsePairConstructor::List(span, results),
                    best_error,
                    pos,
                })
            }
            //To parse a character class, check if the character is accepted, and make an ok/error based on that.
            Constructor::CharacterClass(characters) => {
                let span = Span::from_length(self.file.clone(), pos.position(), 1);
                //TODO clone should not be needed
                if pos.accept(characters.clone()) {
                    Ok(ParseSuccess {
                        result: ParsePairConstructor::Text(span),
                        best_error: None,
                        pos,
                    })
                } else {
                    Err(ParseError::expect_char_class(span, characters.clone()))
                }
            }
            //To parse a choice, try each constructor, keeping track of the best error that occurred while doing so.
            //If none of the constructors succeed, we will return this error.
            Constructor::Choice(constructors) => {
                let mut best_error = None;
                for (i, subconstructor) in constructors.iter().enumerate() {
                    match self.parse_constructor(subconstructor, pos.clone()) {
                        Ok(suc) => {
                            best_error =
                                ParseError::combine_option_parse_error(best_error, suc.best_error);
                            return Ok(ParseSuccess {
                                result: ParsePairConstructor::Choice(
                                    suc.result.span(),
                                    i,
                                    Box::new(suc.result),
                                ),
                                pos: suc.pos,
                                best_error,
                            });
                        }
                        Err(err) => {
                            best_error =
                                ParseError::combine_option_parse_error(best_error, Some(err))
                        }
                    }
                }
                Err(best_error.unwrap())
            }
            //To parse a negative, try parsing the constructor.
            //If it succeeds, we need to make an error, not sure how
            //If it fails, we return ok.
            Constructor::Negative(constructor) => {
                match self.parse_constructor(constructor.as_ref(), pos.clone()) {
                    Ok(_) => {
                        todo!() //Negatives are complicated with errors
                    }
                    Err(err) => {
                        Ok(ParseSuccess {
                            result: ParsePairConstructor::Empty(err.span),
                            best_error: None,
                            pos, //Return old position
                        })
                    }
                }
            }
            //To parse a positive, try parsing the constructor.
            //If it succeeds, we return ok. Otherwise, we return the error.
            Constructor::Positive(constructor) => {
                match self.parse_constructor(constructor.as_ref(), pos.clone()) {
                    Ok(ok) => {
                        Ok(ParseSuccess {
                            result: ParsePairConstructor::Empty(ok.result.span()),
                            best_error: None, //If the positive passed, then we don't care about any "better" parses inside it
                            pos,              //Return old position
                        })
                    }
                    Err(err) => Err(err),
                }
            }
        }
    }
}

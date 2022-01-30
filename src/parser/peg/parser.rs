use crate::parser::error::ParseError;
use crate::parser::peg::parse_success::ParseSuccess;
use crate::parser::syntax_file::ast::{Constructor, Sort, SyntaxFileAst};
use crate::source_file::{SourceFile, SourceFileIterator};
use crate::span::Span;
use std::collections::HashMap;
use crate::parser::peg::parse_pair::{ParsePairConstructor, ParsePairSort};

pub struct ParserState {
    file: SourceFile,
    rules: HashMap<String, Sort>,
}

impl ParserState {
    pub fn parse_file(syntax: SyntaxFileAst, file: SourceFile) -> Result<ParsePairSort, ParseError> {
        let mut state = ParserState {
            file: file.clone(),
            rules: HashMap::new(),
        };
        syntax.sorts.into_iter().for_each(|rule| {
            state.rules.insert(rule.name.clone(), rule).unwrap();
        });

        let mut ok = state.parse_sort(&syntax.starting_rule, file.iter())?;
        if ok.pos.peek().is_none() {
            Ok(ok.result)
        } else {
            match ok.best_error {
                Some(err) => Err(err),
                None => {
                    let curpos = ok.pos.position();
                    while ok.pos.next().is_some() {}
                    let endpos = ok.pos.position();
                    Err(ParseError::NotEntireInput(Span {
                        position: curpos,
                        length: endpos - curpos,
                        source: file,
                    }))
                }
            }
        }
    }

    fn parse_sort<'a>(
        &self,
        rule: &str,
        pos: SourceFileIterator<'a>,
    ) -> Result<ParseSuccess<'a, ParsePairSort>, ParseError> {
        let sort = self.rules.get(rule).unwrap(); //Safe: Names should be defined

        let mut best_error: Option<ParseError> = None;
        for (cname, constructor) in &sort.constructors {
            match self.parse_constructor(constructor, pos.clone()) {
                Ok(ok) => {
                    return Ok(ParseSuccess {
                        result: ParsePairSort {
                            sort: sort.name.clone(),
                            constructor_name: cname.clone(),
                            constructor_value: ok.result,
                        },
                        best_error: ok.best_error.or(best_error),
                        pos,
                    });
                }
                Err(err) => best_error = Self::combine_option_parse_error(best_error, Some(err)),
            }
        }
        Err(best_error.unwrap()) //Safe: Each sort has at least one constructor
    }

    fn parse_constructor<'a>(
        &self,
        constructor: &Constructor,
        mut pos: SourceFileIterator<'a>,
    ) -> Result<ParseSuccess<'a, ParsePairConstructor>, ParseError> {
        match constructor {
            Constructor::Identifier(rule) => {
                //Parse the sort, wrap it in a Sort constructor
                Ok(self.parse_sort(rule, pos)?.map(|s| ParsePairConstructor::Sort(s.span(), Box::new(s))))
            }
            Constructor::Literal(lit) => {
                let span = Span::from_length(self.file.clone(), pos.position(), lit.len());
                if pos.accept_str(lit) {
                    Ok(ParseSuccess {
                        result: ParsePairConstructor::Text(span),
                        best_error: None,
                        pos,
                    })
                } else {
                    Err(ParseError::ExpectString(span, lit.clone()))
                }
            }
            Constructor::Sequence(constructors) => {
                let mut results = vec![];
                let mut best_error = None;
                let start_pos = pos.position();

                for subconstructor in constructors {
                    match self.parse_constructor(subconstructor, pos) {
                        Ok(ok) => {
                            pos = ok.pos;
                            best_error = Self::combine_option_parse_error(best_error, ok.best_error);
                            results.push(ok.result);
                        }
                        Err(err) => {
                            best_error = Self::combine_option_parse_error(best_error, Some(err));
                            return Err(best_error.unwrap());
                        }
                    }
                }
                let span = Span::from_end(self.file.clone(), start_pos, pos.position());
                Ok(ParseSuccess {
                    result: ParsePairConstructor::List(span, results),
                    best_error,
                    pos,
                })
            }
            Constructor::Repeat { c, min, max } => {
                let mut results = vec![];
                let mut best_error = None;
                let start_pos = pos.position();

                for _ in 0..*min {
                    match self.parse_constructor(c.as_ref(), pos) {
                        Ok(ok) => {
                            results.push(ok.result);
                            pos = ok.pos;
                            best_error =
                                Self::combine_option_parse_error(best_error, ok.best_error);
                        }
                        Err(err) => {
                            best_error = Self::combine_option_parse_error(best_error, Some(err));
                            return Err(best_error.unwrap());
                        }
                    }
                }

                for _ in *min..max.unwrap_or(u64::MAX) {
                    match self.parse_constructor(c.as_ref(), pos.clone()) {
                        Ok(ok) => {
                            results.push(ok.result);
                            pos = ok.pos;
                            best_error =
                                Self::combine_option_parse_error(best_error, ok.best_error);
                        }
                        Err(err) => {
                            best_error = Self::combine_option_parse_error(best_error, Some(err));
                            break;
                        }
                    }
                }

                let span = Span::from_end(self.file.clone(), start_pos, pos.position());
                Ok(ParseSuccess {
                    result: ParsePairConstructor::List(span, results),
                    best_error,
                    pos,
                })
            }
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
                    Err(ParseError::ExpectCharClass(span, characters.clone()))
                }
            }
            Constructor::Choice(constructors) => {
                let mut best_error = None;
                for (i, subconstructor) in constructors.into_iter().enumerate() {
                    match self.parse_constructor(subconstructor, pos.clone()) {
                        Ok(suc) => {
                            best_error = Self::combine_option_parse_error(best_error, suc.best_error);
                            return Ok(ParseSuccess { result: ParsePairConstructor::Choice(suc.result.span(), i, Box::new(suc.result)), pos: suc.pos, best_error });
                        }
                        Err(err) => {
                            best_error = Self::combine_option_parse_error(best_error, Some(err))
                        }
                    }
                }
                return Err(best_error.unwrap());
            }
            Constructor::Negative(constructor) => {
                match self.parse_constructor(constructor.as_ref(), pos.clone()) {
                    Ok(_) => {
                        todo!() //Negatives are complicated with errors
                    },
                    Err(err) => {
                        Ok(ParseSuccess {
                            result: ParsePairConstructor::Empty(err.span()),
                            best_error: None,
                            pos, //Return old position
                        })
                    }
                }

            }
            Constructor::Positive(constructor) => {
                match self.parse_constructor(constructor.as_ref(), pos.clone()) {
                    Ok(ok) => {
                        Ok(ParseSuccess {
                            result: ParsePairConstructor::Empty(ok.result.span()),
                            best_error: None, //If the positive passed, then we don't care about any "better" parses inside it
                            pos, //Return old position
                        })
                    }
                    Err(err) => Err(err),
                }
            }
        }
    }

    fn combine_option_parse_error(
        a: Option<ParseError>,
        b: Option<ParseError>,
    ) -> Option<ParseError> {
        match (a, b) {
            (None, None) => None,
            (None, Some(e)) => Some(e),
            (Some(e), None) => Some(e),
            (Some(e1), Some(e2)) => Some(e1.combine(e2)),
        }
    }
}

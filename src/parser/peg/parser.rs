use std::collections::HashMap;
use crate::parser::error::ParseError;
use crate::parser::peg::parse_success::ParseSuccess;
use crate::parser::syntax_file::ast::{Constructor, Sort, SyntaxFileAst};
use crate::source_file::{SourceFile, SourceFileIterator};
use crate::span::Span;

pub struct ParserState {
    file: SourceFile,
    rules: HashMap<String, Sort>,
}

impl ParserState {
    pub fn parse_file<'a>(syntax: SyntaxFileAst, file: SourceFile) -> Result<(), ParseError> {
        let mut state = ParserState {
            file: file.clone(),
            rules: HashMap::new(),
        };
        syntax.sorts.into_iter().for_each(|rule| { state.rules.insert(rule.name.clone(), rule).unwrap(); });


        let mut ok = state.parse_rule(&syntax.starting_rule, file.iter())?;
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
                        source: file
                    }))
                }
            }
        }
    }

    fn parse_rule<'a>(&self, rule: &str, pos: SourceFileIterator<'a>) -> Result<ParseSuccess<'a, ()>, ParseError> {
        let sort = self.rules.get(rule).unwrap(); //Safe: Names should be defined

        let mut best_error: Option<ParseError> = None;
        for constructor in &sort.constructors {
            match self.parse_constructor(constructor, pos.clone()) {
                Ok(ok) => return Ok(ParseSuccess {
                    result: (),
                    best_error: ok.best_error.or(best_error),
                    pos
                }),
                Err(err) => {
                    best_error = Self::combine_option_parse_error(best_error, Some(err))
                }
            }
        }
        return Err(best_error.unwrap()); //Safe: Each sort has at least one constructor
    }

    fn parse_constructor<'a>(&self, constructor: &Constructor, mut pos: SourceFileIterator<'a>) -> Result<ParseSuccess<'a, ()>, ParseError> {
        match constructor {
            Constructor::Identifier(rule) => {
                self.parse_rule(rule, pos)
            }
            Constructor::Literal(lit) => {
                if pos.accept_str(lit) {
                    Ok(ParseSuccess {
                        result: (),
                        best_error: None,
                        pos
                    })
                } else {
                    Err(ParseError::ExpectString(
                        Span{
                            position: pos.position(),
                            length: lit.len(),
                            source: self.file.clone(),
                        },
                        lit.clone(),
                    ) )
                }
            }
            Constructor::Sequence(constructors) => {
                let mut results = vec![];
                let mut best_error = None;
                for subconstructor in constructors {
                    match self.parse_constructor(subconstructor, pos) {
                        Ok(ok) => {
                            pos = ok.pos;
                            best_error = Self::combine_option_parse_error(best_error, ok.best_error);
                            results.push(ok.result);
                        }
                        Err(err) => {
                            best_error = Self::combine_option_parse_error(best_error, Some(err));
                            return Err(best_error.unwrap())
                        }
                    }
                }
                Ok(ParseSuccess {
                    result: (),
                    best_error,
                    pos
                })
            }
            Constructor::Repeat { c, min, max } => {
                let mut result = vec![];
                let mut best_error = None;

                for _ in 0..*min {
                    match self.parse_constructor(c.as_ref(), pos) {
                        Ok(ok) => {
                            result.push(ok.result);
                            pos = ok.pos;
                            best_error = Self::combine_option_parse_error(best_error, ok.best_error);
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
                            result.push(ok.result);
                            pos = ok.pos;
                            best_error = Self::combine_option_parse_error(best_error, ok.best_error);
                        }
                        Err(err) => {
                            best_error = Self::combine_option_parse_error(best_error, Some(err));
                            break;
                        }
                    }
                }

                Ok(ParseSuccess {
                    result: (),
                    best_error,
                    pos
                })
            }
            Constructor::CharacterClass(characters) => {
                if pos.accept(characters.clone() ) { //TODO clone should not be needed
                    Ok(ParseSuccess {
                        result: (),
                        best_error: None,
                        pos
                    })
                } else {
                    Err(ParseError::ExpectCharClass(
                        Span{
                            position: pos.position(),
                            length: 1,
                            source: self.file.clone(),
                        },
                        characters.clone(),
                    ))
                }
            }
            Constructor::Choice(choices) => {
                todo!()
            }
            Constructor::Negative(_) => {
                todo!()
            }
            Constructor::Positive(_) => {
                todo!()
            }
        }
    }

    fn combine_option_parse_error(a: Option<ParseError>, b: Option<ParseError>) -> Option<ParseError> {
        match (a, b) {
            (None, None) => None,
            (None, Some(e)) => Some(e),
            (Some(e), None) => Some(e),
            (Some(e1), Some(e2)) => Some(e1.combine(e2))
        }
    }
}
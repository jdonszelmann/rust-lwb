use std::num::ParseIntError;
use std::str::FromStr;
use crate::codegen_prelude::AstInfo;
use crate::parser::syntax_file::ast;
use crate::parser::bootstrap::ast::{Annotation, Constructor, Expression, Sort, SyntaxFileAst};
use crate::parser::syntax_file::ast::{CharacterClassItem, EscapeClosingBracket, Identifier, Meta, Number, SortOrMeta};
use crate::sources::character_class::CharacterClass;
use thiserror::Error;
use crate::parser::syntax_file::convert_syntax_file_ast::AstConversionError::DuplicateStartingRule;

#[derive(Debug, Error)]
pub enum AstConversionError {
    #[error("grammar contains more than one starting rule")]
    DuplicateStartingRule,

    #[error("couldn't parse int: {0}")]
    NumberConversion(ParseIntError),

    #[error("{0} is not a valid annotation")]
    BadAnnotation(String),
}

pub type ConversionResult<T> = Result<T, AstConversionError>;

pub fn convert_syntax_file_ast<M: AstInfo>(inp: ast::AST_ROOT<M>) -> ConversionResult<SyntaxFileAst> {
    match inp {
        ast::Program::Program(_, sort_or_metas) => {
            let mut layout = CharacterClass::Nothing;
            let mut sorts = Vec::new();
            let mut start = None;

            for i in sort_or_metas {
                match *i {
                    SortOrMeta::Meta(_, m) => {
                        match *m {
                            Meta::Layout(_, l) => {
                                layout = layout.combine(convert_character_class(*l)?)
                            }
                            Meta::Start(_, name) => {
                                if start.is_some() {
                                    return Err(DuplicateStartingRule);
                                } else {
                                    start = Some(convert_identifier(*name))
                                }
                            }
                        }
                    }
                    SortOrMeta::Sort(_, sort) => {
                        sorts.push(convert_sort(*sort)?)
                    }
                }
            }

            Ok(SyntaxFileAst {
                sorts,
                starting_sort: "".to_string(),
                layout: CharacterClass::Nothing,
            })
        }
    }
}

fn convert_identifier<M: AstInfo>(inp: ast::Identifier<M>) -> String {
    match inp {
        Identifier::Identifier(_, name) => name,
    }
}

fn convert_number<M: AstInfo>(inp: ast::Number<M>) -> ConversionResult<u64> {
    match inp {
        Number::Number(_, name) => {
            name.parse::<u64>().map_err(AstConversionError::NumberConversion)
        }
    }
}

fn convert_escape_closing_bracket<M: AstInfo>(inp: ast::EscapeClosingBracket<M>) -> char {
   match inp {
       EscapeClosingBracket::Escaped(_, c) => {
           match c.as_str() {
               "n" => '\n',
               "r" => '\r',
               "t" => '\t',
               "\\" => '\\',
               "]" => ']',
               a => unreachable!("grammar shouldn't allow {} here", a),
           }
       }
       EscapeClosingBracket::Unescaped(_, c) => {
           let c = c.chars().next().expect("only one character");
           c
       }
   }
}

fn convert_character_class<M: AstInfo>(inp: ast::CharacterClass<M>) -> ConversionResult<CharacterClass> {
    Ok(match inp {
        ast::CharacterClass::Class(_, inverted, items) => {
            let mut res = CharacterClass::Nothing;

            for i in items {
                match *i {
                    CharacterClassItem::Range(_, from, to_inclusive) => {
                        res = res.combine(CharacterClass::RangeInclusive {
                            from: convert_escape_closing_bracket(*from),
                            to: convert_escape_closing_bracket(*to_inclusive),
                        })
                    }
                    CharacterClassItem::SingleChar(_, c) => {
                        let c = convert_escape_closing_bracket(*c);

                        res = res.combine(CharacterClass::RangeInclusive {
                            from: c,
                            to: c,
                        })
                    }
                }
            }


            if inverted {
                res = res.invert();
            }

            res
        }
    })
}

fn convert_sort<M: AstInfo>(inp: ast::Sort<M>) -> ConversionResult<Sort> {
    Ok(match inp {
        ast::Sort::Sort(_, name, constructors) => {
            Sort {
                name: convert_identifier(*name),
                constructors: constructors.into_iter().map(|i| convert_constructor(*i)).collect::<Result<_, _>>()?,
            }
        }
    })
}

fn convert_expression<M: AstInfo>(inp: ast::Expression<M>) -> ConversionResult<Expression> {
    Ok(match inp {
        ast::Expression::Star(_, exp) => Expression::Repeat {
            c: Box::new(convert_expression(*exp)?),
            min: 0,
            max: None,
        },
        ast::Expression::Plus(_, exp) => Expression::Repeat {
            c: Box::new(convert_expression(*exp)?),
            min: 1,
            max: None,
        },
        ast::Expression::Maybe(_, exp) => Expression::Repeat {
            c: Box::new(convert_expression(*exp)?),
            min: 0,
            max: Some(1),
        },
        ast::Expression::RepeatExact(_, exp, min, max) => Expression::Repeat {
            c: Box::new(convert_expression(*exp)?),
            min: convert_number(*min)?,
            max: max.map(|i| convert_number(*i)).transpose()?,
        },
        ast::Expression::Literal(_, l) => Expression::Literal(l),
        ast::Expression::Sort(_, s) => Expression::Sort(convert_identifier(*s)),
        ast::Expression::Class(_, cc) => Expression::CharacterClass(convert_character_class(*cc)?),
        ast::Expression::Paren(_, exp) => convert_expressions(exp)?,
    })
}

fn convert_expressions<M: AstInfo>(mut inp: Vec<Box<ast::Expression<M>>>) -> ConversionResult<Expression> {
    // 0 not possible due to grammar constraints
    if inp.len() == 1 {
        convert_expression(*inp.pop().unwrap())
    } else {
        Ok(Expression::Sequence(inp.into_iter().map(|i| convert_expression(*i)).collect::<Result<_, _>>()?))
    }
}

fn convert_annotations<M: AstInfo>(inp: ast::Annotation<M>) -> ConversionResult<Vec<Annotation>> {
    match inp {
        ast::Annotation::Annotation(_, first, rest, _) => {
            let mut res = Vec::new();

            if let Some(i) = first {
                res.push(convert_identifier(*i));
            }

            for i in rest {
                res.push(convert_identifier(*i));
            }

            Ok(res.into_iter()
                .map(|i| {
                    Annotation::from_str(&i)
                        .map_err(|_| AstConversionError::BadAnnotation(i.clone()))
                })
                .collect::<Result<_, _>>()?
            )
        }
    }
}

fn convert_constructor<M: AstInfo>(inp: ast::Constructor<M>) -> ConversionResult<Constructor> {
    match inp {
        ast::Constructor::Constructor(_, name, expressions, annotations) => {
            Ok(Constructor {
                name: convert_identifier(*name),
                expression: convert_expressions(expressions)?,
                annotations: vec![],
            })
        }
    }
}

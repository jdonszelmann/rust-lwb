use crate::codegen_prelude::AstInfo;
use crate::parser::peg::parser_sugar_ast::*;
use crate::parser::syntax_file::ast;
use crate::parser::syntax_file::ast::{CharacterClassItem, EscapeClosingBracket, SortOrMeta};
use crate::parser::syntax_file::convert_syntax_file_ast::AstConversionError::{
    DuplicateStartingRule, NoStartingSort,
};
use crate::parser::syntax_file::AST::{DelimitedBound, StringChar};
use crate::sources::character_class::CharacterClass;
use std::num::ParseIntError;
use std::str::FromStr;
use itertools::Itertools;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AstConversionError {
    #[error("grammar contains more than one starting rule")]
    DuplicateStartingRule,

    #[error("couldn't parse int: {0}")]
    NumberConversion(ParseIntError),

    #[error("{0} is not a valid annotation")]
    BadAnnotation(String),

    #[error("no starting sort in syntax file definition")]
    NoStartingSort,
}

pub type ConversionResult<T> = Result<T, AstConversionError>;

pub fn convert<M: AstInfo>(inp: ast::AST_ROOT<M>) -> ConversionResult<SyntaxFileAst> {
    let ast::Program(_, sort_or_metas) = inp;
    let mut sorts = Vec::new();
    let mut start = None;

    for i in sort_or_metas {
        match *i {
            SortOrMeta::Meta(_, m) => {
                if start.is_some() {
                    return Err(DuplicateStartingRule);
                } else {
                    start = Some(convert_identifier(*m.1))
                }
            }
            SortOrMeta::Sort(_, sort) => sorts.push(convert_sort(*sort)?),
        }
    }

    Ok(SyntaxFileAst {
        sorts,
        starting_sort: start.ok_or(NoStartingSort)?,
    })
}

fn convert_identifier<M: AstInfo>(inp: ast::Identifier<M>) -> String {
    inp.1.trim().to_string()
}

fn convert_number<M: AstInfo>(inp: ast::Number<M>) -> ConversionResult<u64> {
    inp.1
        .parse::<u64>()
        .map_err(AstConversionError::NumberConversion)
}

fn convert_escape_closing_bracket<M: AstInfo>(inp: ast::EscapeClosingBracket<M>) -> char {
    match inp {
        EscapeClosingBracket::Escaped(_, c) => match c.as_str() {
            "n" => '\n',
            "r" => '\r',
            "t" => '\t',
            "\\" => '\\',
            "]" => ']',
            a => unreachable!("grammar shouldn't allow {} here", a),
        },
        EscapeClosingBracket::Unescaped(_, c) => {
            let c = c.chars().next().expect("only one character");
            c
        }
    }
}

fn convert_string_char<M: AstInfo>(inp: ast::StringChar<M>) -> char {
    match inp {
        StringChar::Escaped(_, c) => match c.as_str() {
            "n" => '\n',
            "r" => '\r',
            "t" => '\t',
            "\\" => '\\',
            "\"" => '"',
            a => unreachable!("grammar shouldn't allow {} here", a),
        },
        StringChar::Normal(_, c) => {
            let c = c.chars().next().expect("only one character");
            c
        }
    }
}

fn convert_character_class<M: AstInfo>(
    inp: ast::CharacterClass<M>,
) -> ConversionResult<CharacterClass> {
    let ast::CharacterClass(_, inverted, items) = inp;
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

                res = res.combine(CharacterClass::RangeInclusive { from: c, to: c })
            }
        }
    }

    if inverted {
        res = res.invert();
    }

    Ok(res)
}

fn convert_sort<M: AstInfo>(inp: ast::Sort<M>) -> ConversionResult<Sort> {
    Ok(match inp {
        ast::Sort::Sort(_, name, constructors) => Sort {
            name: convert_identifier(*name),
            constructors: constructors
                .into_iter()
                .map(|i| convert_constructor(*i))
                .collect::<Result<_, _>>()?,
        },
        ast::Sort::SortSingle(_, name, expressions, annotations) => {
            let name = convert_identifier(*name);
            Sort {
                name: name.clone(),
                constructors: vec![Constructor {
                    name,
                    expression: convert_expressions(expressions)?,
                    annotations: if let Some(a) = annotations {
                        convert_annotations(*a)?
                    } else {
                        Vec::new()
                    },
                }],
            }
        }
    })
}

fn convert_expression<M: AstInfo>(inp: ast::Expression<M>) -> ConversionResult<Expression> {
    Ok(match inp {
        ast::Expression::Star(_, exp) => Expression::Repeat {
            e: Box::new(convert_expression(*exp)?),
            min: 0,
            max: None,
        },
        ast::Expression::Plus(_, exp) => Expression::Repeat {
            e: Box::new(convert_expression(*exp)?),
            min: 1,
            max: None,
        },
        ast::Expression::Maybe(_, exp) => Expression::Repeat {
            e: Box::new(convert_expression(*exp)?),
            min: 0,
            max: Some(1),
        },
        ast::Expression::RepeatExact(_, exp, min, max) => Expression::Repeat {
            e: Box::new(convert_expression(*exp)?),
            min: convert_number(*min)?,
            max: max.map(|i| convert_number(*i)).transpose()?,
        },
        ast::Expression::Literal(_, l) | ast::Expression::SingleQuoteLiteral(_, l) => {
            Expression::Literal(l.into_iter().map(|i| convert_string_char(*i)).collect())
        }
        ast::Expression::Sort(_, s) => Expression::Sort(convert_identifier(*s)),
        ast::Expression::Class(_, cc) => Expression::CharacterClass(convert_character_class(*cc)?),
        ast::Expression::Paren(_, exp) => convert_expressions(exp)?,
        ast::Expression::Delimited(_, exp, delim, bound, trailing) => {
            let (min, max) = match *bound {
                DelimitedBound::NumNum(_, min, max) => {
                    (convert_number(*min)?, Some(convert_number(*max)?))
                }
                DelimitedBound::NumInf(_, min) => (convert_number(*min)?, None),
                DelimitedBound::Num(_, min) => (convert_number(*min)?, None),
                DelimitedBound::Star(_) => (0, None),
                DelimitedBound::Plus(_) => (1, None),
            };
            Expression::Delimited {
                e: Box::new(convert_expression(*exp)?),
                delim: Box::new(convert_expression(*delim)?),
                min,
                max,
                trailing,
            }
        }
    })
}

fn convert_expressions<M: AstInfo>(
    mut inp: Vec<Box<ast::Expression<M>>>,
) -> ConversionResult<Expression> {
    // 0 not possible due to grammar constraints
    if inp.len() == 1 {
        convert_expression(*inp.pop().unwrap())
    } else {
        Ok(Expression::Sequence(
            inp.into_iter()
                .map(|i| convert_expression(*i))
                .collect::<Result<_, _>>()?,
        ))
    }
}

fn convert_annotations<M: AstInfo>(inp: ast::Annotation<M>) -> ConversionResult<Vec<Annotation>> {
    let ast::Annotation(_, annotations) = inp;
    annotations.into_iter()
        .map(|an| Annotation::from_str(&an.as_ref().1).map_err(|_| AstConversionError::BadAnnotation(an.as_ref().1.clone())))
        .collect::<Result<_, _>>()
}

fn convert_constructor<M: AstInfo>(inp: ast::Constructor<M>) -> ConversionResult<Constructor> {
    let ast::Constructor(_, name, expressions, annotations) = inp;
    Ok(Constructor {
        name: convert_identifier(*name),
        expression: convert_expressions(expressions)?,
        annotations: if let Some(a) = annotations {
            convert_annotations(*a)?
        } else {
            Vec::new()
        },
    })
}

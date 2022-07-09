use crate::codegen_prelude::AstInfo;
use crate::parser::peg::parser_sugar_ast::*;
use crate::parser::syntax_file::ast;
use crate::parser::syntax_file::ast::{CharacterClassItem, EscapeClosingBracket, SortOrMeta};
use crate::parser::syntax_file::convert_syntax_file_ast::AstConversionError::{
    DuplicateStartingRule, NoStartingSort,
};
use crate::parser::syntax_file::AST::{DelimitedBound, StringChar};
use crate::sources::character_class::CharacterClass;
use std::collections::HashMap;
use std::num::ParseIntError;
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
    let mut sorts = HashMap::new();
    let mut start = None;

    for i in sort_or_metas {
        match i {
            SortOrMeta::Meta(_, m) => {
                if start.is_some() {
                    return Err(DuplicateStartingRule);
                } else {
                    start = Some(convert_identifier(m.1))
                }
            }
            SortOrMeta::Sort(_, sort) => {
                let converted = convert_sort(sort)?;
                sorts.insert(converted.name.clone(), converted);
            }
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
        match i {
            CharacterClassItem::Range(_, from, to_inclusive) => {
                res = res.combine(CharacterClass::RangeInclusive {
                    from: convert_escape_closing_bracket(from),
                    to: convert_escape_closing_bracket(to_inclusive),
                })
            }
            CharacterClassItem::SingleChar(_, c) => {
                let c = convert_escape_closing_bracket(c);

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
        ast::Sort::Sort(_, name, annos, constructors) => Sort {
            documentation: None,
            name: convert_identifier(name),
            constructors: constructors
                .into_iter()
                .map(|i| convert_constructor(i))
                .collect::<Result<_, _>>()?,
            annotations: annos.map_or(Ok(vec![]), |a| convert_annotations(&a))?
        },
        ast::Sort::SortSingle(_, name, expressions, annotations) => {
            let name = convert_identifier(name);
            Sort {
                documentation: None,
                name: name.clone(),
                constructors: vec![Constructor {
                    documentation: None,
                    name,
                    expression: convert_expressions(expressions)?,
                    annotations: annotations.as_ref().map_or(Ok(vec![]), |a| convert_annotations(a))?,
                }],
                annotations: annotations.as_ref().map_or(Ok(vec![]), |a| convert_annotations(a))?

            }
        }
        ast::Sort::SortDocumented(_, comments, sort) => convert_sort(*sort).and_then(|mut i| {
            i.documentation = Some(convert_comments(comments)?);
            Ok(i)
        })?,
    })
}

fn convert_comments<M: AstInfo>(inp: Vec<ast::DocComment<M>>) -> ConversionResult<String> {
    Ok(inp
        .into_iter()
        .map(|i| i.1.strip_prefix("///").unwrap_or(&i.1).trim().to_string())
        .collect::<Vec<_>>()
        .join("\n"))
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
        ast::Expression::RepeatExact(_, exp, num) => {
            let converted_num = convert_number(num)?;
            Expression::Repeat {
                e: Box::new(convert_expression(*exp)?),
                min: converted_num,
                max: Some(converted_num),
            }
        }
        ast::Expression::RepeatRange(_, exp, min, max) => Expression::Repeat {
            e: Box::new(convert_expression(*exp)?),
            min: convert_number(min)?,
            max: Some(convert_number(max)?),
        },
        ast::Expression::RepeatLower(_, exp, num) => Expression::Repeat {
            e: Box::new(convert_expression(*exp)?),
            min: convert_number(num)?,
            max: None,
        },
        ast::Expression::Literal(_, l) | ast::Expression::SingleQuoteLiteral(_, l) => {
            Expression::Literal(l.into_iter().map(|i| convert_string_char(i)).collect())
        }
        ast::Expression::Sort(_, s) => Expression::Sort(convert_identifier(s)),
        ast::Expression::Class(_, cc) => Expression::CharacterClass(convert_character_class(cc)?),
        ast::Expression::Paren(_, exp) => {
            convert_expressions(exp.into_iter().map(|i| *i).collect())?
        }
        ast::Expression::Delimited(_, exp, delim, bound, trailing) => {
            let (min, max) = match bound {
                DelimitedBound::NumNum(_, min, max) => {
                    (convert_number(min)?, Some(convert_number(max)?))
                }
                DelimitedBound::NumInf(_, min) => (convert_number(min)?, None),
                DelimitedBound::Num(_, min) => (convert_number(min)?, None),
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
    mut inp: Vec<ast::Expression<M>>,
) -> ConversionResult<Expression> {
    // 0 not possible due to grammar constraints
    if inp.len() == 1 {
        convert_expression(inp.pop().unwrap())
    } else {
        Ok(Expression::Sequence(
            inp.into_iter()
                .map(|i| convert_expression(i))
                .collect::<Result<_, _>>()?,
        ))
    }
}

fn convert_annotations<M: AstInfo>(inp: &ast::AnnotationList<M>) -> ConversionResult<Vec<Annotation>> {
    let ast::AnnotationList(_, annotations) = inp;
    annotations
        .into_iter()
        .map(|an: &ast::Annotation<M>| Ok(match an {
            ast::Annotation::Injection(_) => Annotation::Injection,
            ast::Annotation::NoPrettyPrint(_) => Annotation::NoPrettyPrint,
            ast::Annotation::SingleString(_) => Annotation::SingleString,
            ast::Annotation::NoLayout(_) => Annotation::NoLayout,
            ast::Annotation::Hidden(_) => Annotation::Hidden,
        }))
        .collect::<Result<_, _>>()
}

fn convert_constructor<M: AstInfo>(inp: ast::Constructor<M>) -> ConversionResult<Constructor> {
    Ok(match inp {
        ast::Constructor::Constructor(_, name, expressions, annotations) => Constructor {
            documentation: None,
            name: convert_identifier(name),
            expression: convert_expressions(expressions)?,
            annotations: if let Some(a) = annotations {
                convert_annotations(&a)?
            } else {
                Vec::new()
            },
        },
        ast::Constructor::ConstructorDocumented(_, comments, constructor) => {
            convert_constructor(*constructor).and_then(|mut i| {
                i.documentation = Some(convert_comments(comments)?);
                Ok(i)
            })?
        }
    })
}

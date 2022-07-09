use crate::codegen_prelude::{ParsePairExpression, ParsePairSort};
use crate::parser::peg::parse_error::PEGParseError;
use crate::parser::peg::parser_core_ast::{CoreAst, CoreExpression, CoreSort, ParsePairRaw};
use crate::parser::peg::parser_core_file;
use crate::parser::peg::parser_sugar_ast::{Annotation, Expression, Sort, SyntaxFileAst};
use crate::sources::character_class::CharacterClass;
use crate::sources::source_file::SourceFile;
use itertools::Itertools;
use std::collections::HashMap;

/// Parse a file by:
/// 1. Desugaring the AST to core syntax
/// 2. Parsing the source file using core syntax
/// 3. Resugaring the resulting ParsePairRaw
pub fn parse_file<'src>(
    ast: &'src SyntaxFileAst,
    file: &'src SourceFile,
) -> (ParsePairSort<'src>, Vec<PEGParseError>) {
    //Desugar
    let core_ast = desugar_ast(ast);

    //Parse
    let (res, errs) = parser_core_file::parse_file(&core_ast, file);

    //Resugar
    let starting_sort = ast.sorts.get(&ast.starting_sort).unwrap();
    (resugar_sort(ast, starting_sort, res), errs)
}

fn desugar_ast(ast: &SyntaxFileAst) -> CoreAst {
    let mut sorts = HashMap::new();
    //Insert all sorts
    ast.sorts.values().for_each(|s| {
        sorts.insert(&s.name[..], desugar_sort(s));
    });
    //If there is no layout sort, insert one
    if !sorts.contains_key("layout") {
        sorts.insert(
            "layout",
            CoreSort { name: "layout", expr: CoreExpression::CharacterClass(CharacterClass::Nothing), annotations: vec![]},
        );
    }

    CoreAst {
        sorts,
        starting_sort: &ast.starting_sort,
    }
}

fn desugar_sort(sort: &Sort) -> CoreSort {
    CoreSort {
        name: &sort.name,
        expr: CoreExpression::Choice(
            sort.constructors
                .iter()
                .map(|c| {
                    let mut base = desugar_expr(&c.expression);
                    if c.annotations.contains(&Annotation::NoLayout) {
                        base = CoreExpression::FlagNoLayout(Box::new(base));
                        base = CoreExpression::FlagNoErrors(
                            Box::new(base),
                            String::from_iter([&sort.name, ".", &c.name]),
                        );
                    }
                    base
                })
                .collect(),
        ),
        annotations: sort.annotations.clone(),
    }
}

fn desugar_expr(expr: &Expression) -> CoreExpression {
    match expr {
        Expression::Sort(name) => CoreExpression::Name(&name[..]),
        Expression::Sequence(constructors) => {
            CoreExpression::Sequence(constructors.iter().map(desugar_expr).collect_vec())
        }
        Expression::Repeat { e: c, min, max } => CoreExpression::Repeat {
            subexpr: Box::new(desugar_expr(c)),
            min: *min,
            max: *max,
        },
        Expression::CharacterClass(cc) => CoreExpression::CharacterClass(cc.clone()),
        Expression::Choice(constructors) => {
            CoreExpression::Choice(constructors.iter().map(desugar_expr).collect_vec())
        }
        //Literals are desugared to a sequence of character classes
        Expression::Literal(lit) => {
            CoreExpression::FlagNoLayout(Box::new(CoreExpression::FlagNoErrors(
                Box::new(CoreExpression::Sequence(
                    lit.chars()
                        .map(|c| CoreExpression::CharacterClass(c.into()))
                        .collect_vec(),
                )),
                String::from_iter(["'", lit, "'"]),
            )))
        }
        Expression::Negative(_) => {
            todo!()
        }
        Expression::Positive(_) => {
            todo!()
        }
        Expression::Delimited {
            e,
            delim,
            min,
            max,
            trailing,
        } => {
            let e = desugar_expr(e);
            let delim = desugar_expr(delim);

            let mut options = vec![];
            //Can parse count > 0
            if max.is_none() || max.unwrap() > 0 {
                options.push(CoreExpression::Sequence(vec![
                    e.clone(),
                    CoreExpression::Repeat {
                        subexpr: Box::new(CoreExpression::Sequence(vec![delim.clone(), e.clone()])),
                        min: min.saturating_sub(1),
                        max: max.map(|max| max.saturating_sub(1)),
                    },
                ]));
            }
            //Can parse count == 0
            if *min == 0 {
                options.push(CoreExpression::Sequence(vec![]));
            }

            let choice = CoreExpression::Choice(options);
            if *trailing {
                CoreExpression::Sequence(vec![
                    choice,
                    CoreExpression::Repeat {
                        subexpr: Box::new(delim),
                        min: 0,
                        max: Some(1),
                    },
                ])
            } else {
                CoreExpression::Sequence(vec![choice])
            }
        }
    }
}

fn resugar_sort<'src>(
    ast: &'src SyntaxFileAst,
    sort: &'src Sort,
    pair: ParsePairRaw,
) -> ParsePairSort<'src> {
    match pair {
        ParsePairRaw::Choice(_, i, subpair) => ParsePairSort {
            sort: &sort.name[..],
            constructor_name: &sort.constructors[i].name[..],
            constructor_value: resugar_expr(ast, &sort.constructors[i].expression, *subpair),
        },
        ParsePairRaw::Error(span) => ParsePairSort {
            sort: &sort.name[..],
            constructor_name: "ERROR",
            constructor_value: ParsePairExpression::Error(span),
        },
        _ => unreachable!(),
    }
}

fn resugar_expr<'src>(
    ast: &'src SyntaxFileAst,
    sort: &'src Expression,
    pair: ParsePairRaw,
) -> ParsePairExpression<'src> {
    match (sort, pair) {
        (Expression::Sort(name), ParsePairRaw::Name(span, val)) => ParsePairExpression::Sort(
            span,
            Box::new(resugar_sort(ast, ast.sorts.get(name).unwrap(), *val)),
        ),
        (Expression::Sequence(exprs), ParsePairRaw::List(span, vals)) => ParsePairExpression::List(
            span,
            exprs
                .iter()
                .zip(vals.into_iter())
                .map(|(e, v)| resugar_expr(ast, e, v))
                .collect_vec(),
        ),
        (Expression::Repeat { e: c, .. }, ParsePairRaw::List(span, vals)) => {
            ParsePairExpression::List(
                span,
                vals.into_iter()
                    .map(|v| resugar_expr(ast, c, v))
                    .collect_vec(),
            )
        }
        (Expression::CharacterClass(_), ParsePairRaw::Empty(span)) => {
            ParsePairExpression::Empty(span)
        }
        (Expression::Choice(constructors), ParsePairRaw::Choice(span, i, expr)) => {
            ParsePairExpression::Choice(
                span,
                i,
                Box::new(resugar_expr(ast, &constructors[i], *expr)),
            )
        }
        (Expression::Literal(_), ParsePairRaw::List(span, _)) => ParsePairExpression::Empty(span),
        (Expression::Delimited { e, max, .. }, ParsePairRaw::List(span, list)) => {
            //If max is 0, empty list
            if !max.is_none() && max.unwrap() == 0 {
                return ParsePairExpression::List(span, vec![]);
            };
            //Get choice
            let (i, choice) =
                if let ParsePairRaw::Choice(_, i, choice) = list.into_iter().next().unwrap() {
                    (i, choice)
                } else {
                    return ParsePairExpression::Error(span);
                };
            //If choice was not 0, empty list
            if i != 0 {
                return ParsePairExpression::List(span, vec![]);
            };
            //Find elements inside choice
            let seq = if let ParsePairRaw::List(_, seq) = *choice {
                seq
            } else {
                return ParsePairExpression::Error(span);
            };

            let mut result = vec![];
            let mut seq_iter = seq.into_iter();

            //Inside choice is first an expr, then a repeat of seq (delim, expr)
            //We first find the first expr
            let seq0 = seq_iter.next().unwrap();
            result.push(resugar_expr(ast, e, seq0));

            //See if the rest of the expr is present
            let next = seq_iter.next();
            if next.is_none() {
                return ParsePairExpression::List(span, result);
            }
            //It is present, lets get the list of them
            let seq1 = if let ParsePairRaw::List(_, list) = next.unwrap() {
                list
            } else {
                return ParsePairExpression::Error(span);
            };
            //Map each element in the list to get the expr
            seq1.into_iter().for_each(|pair| {
                result.push(if let ParsePairRaw::List(span, list) = pair {
                    if list.len() < 2 {
                        ParsePairExpression::Error(span)
                    } else {
                        resugar_expr(ast, e, list.into_iter().nth(1).unwrap())
                    }
                } else {
                    ParsePairExpression::Error(pair.span())
                });
            });

            ParsePairExpression::List(span, result)
        }
        (_, ParsePairRaw::Error(span)) => ParsePairExpression::Error(span),
        (_, _) => unreachable!(),
    }
}

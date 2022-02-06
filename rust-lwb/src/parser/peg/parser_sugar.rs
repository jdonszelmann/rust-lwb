use crate::codegen_prelude::{ParsePairExpression, ParsePairSort};
use crate::parser::peg::parse_error::PEGParseError;
use crate::parser::peg::parser_core_ast::{CoreAst, CoreExpression, ParsePairRaw};
use crate::parser::peg::parser_core_file;
use crate::parser::peg::parser_sugar_ast::{Annotation, Expression, Sort, SyntaxFileAst};
use crate::sources::source_file::SourceFile;
use itertools::Itertools;
use std::collections::HashMap;

pub fn parse_file<'src>(
    ast: &'src SyntaxFileAst,
    file: &'src SourceFile,
) -> (ParsePairSort<'src>, Vec<PEGParseError>) {
    let core_ast = desugar_ast(ast);
    let (res, errs) = parser_core_file::parse_file(&core_ast, file);

    let starting_sort = ast
        .sorts
        .iter()
        .find(|s| s.name == ast.starting_sort)
        .unwrap();
    (resugar_sort(ast, starting_sort, res), errs)
}

fn desugar_ast(ast: &SyntaxFileAst) -> CoreAst {
    let mut sorts = HashMap::new();
    ast.sorts.iter().for_each(|s| {
        sorts.insert(&s.name[..], desugar_sort(s));
    });
    CoreAst {
        sorts,
        starting_sort: &ast.starting_sort,
        layout: ast.layout.clone(),
    }
}

fn desugar_sort(sort: &Sort) -> CoreExpression {
    CoreExpression::Choice(
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
    )
}

fn desugar_expr(expr: &Expression) -> CoreExpression {
    match expr {
        Expression::Sort(name) => CoreExpression::Name(&name[..]),
        Expression::Sequence(constructors) => {
            CoreExpression::Sequence(constructors.iter().map(desugar_expr).collect_vec())
        }
        Expression::Repeat { c, min, max } => CoreExpression::Repeat {
            subexpr: Box::new(desugar_expr(c)),
            min: *min,
            max: *max,
        },
        Expression::CharacterClass(cc) => CoreExpression::CharacterClass(cc.clone()),
        Expression::Choice(constructors) => {
            CoreExpression::Choice(constructors.iter().map(desugar_expr).collect_vec())
        }
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
            Box::new(resugar_sort(
                ast,
                ast.sorts.iter().find(|s| s.name == *name).unwrap(),
                *val,
            )),
        ),
        (Expression::Sequence(exprs), ParsePairRaw::List(span, vals)) => ParsePairExpression::List(
            span,
            exprs
                .iter()
                .zip(vals.into_iter())
                .map(|(e, v)| resugar_expr(ast, e, v))
                .collect_vec(),
        ),
        (Expression::Repeat { c, .. }, ParsePairRaw::List(span, vals)) => {
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
        (_, ParsePairRaw::Error(span)) => ParsePairExpression::Error(span),
        (_, _) => unreachable!(),
    }
}

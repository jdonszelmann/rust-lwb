use std::collections::HashMap;
use crate::codegen::check_recursive::{BreadthFirstAstIterator, RecursionChecker};
use crate::codegen::error::CodegenError;
use crate::codegen::sanitize_identifier;
use crate::parser::peg::parser_sugar_ast::Annotation::SingleString;
use crate::parser::peg::parser_sugar_ast::{Annotation, Expression, Sort, SyntaxFileAst};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

fn generate_unpack_expression(
    expression: &Expression,
    sort: &str,
    src: TokenStream,
    ckr: &RecursionChecker,
    non_exhaustive: TokenStream,
    sort_list: &HashMap<&str, &Sort>
) -> Option<TokenStream> {
    let unreachable_exp = quote!(unreachable!("expected different parse pair expression in pair to ast conversion of {}", #sort););

    Some(match expression {
        Expression::Sort(name) => {
            if sort_list.get(name.as_str()).map(|i| i.annotations.contains(&Annotation::Hidden)).unwrap_or_default() {
                return None;
            }

            let iname = format_ident!("{}", sanitize_identifier(name));

            let inner = ckr.maybe_box(
                name,
                quote!(
                    #iname::from_pairs(s, generator)
                ),
            );

            quote!(
                if let ParsePairExpression::Sort(_, ref s) = #src {
                    #inner
                } else { #unreachable_exp }
            )
        }
        Expression::CharacterClass(_) => {
            quote!(
                if let ParsePairExpression::Empty(ref span) = #src {
                    span.as_str().to_string()
                } else { #unreachable_exp }
            )
        }
        Expression::Repeat { min, max, e } | Expression::Delimited { min, max, e, .. } => {
            if let Some(ue) = generate_unpack_expression(e, sort, quote!(x), ckr, non_exhaustive, sort_list) {
                match (min, max) {
                    (0, Some(1)) => quote!(
                        if let ParsePairExpression::List(_, ref l) = #src {
                            l.first().map(|x| #ue)
                        } else { #unreachable_exp }
                    ),
                    _ => quote!(
                        if let ParsePairExpression::List(_, ref l) = #src {
                            l.iter().map(|x| #ue).collect()
                        } else { #unreachable_exp }
                    ),
                }
            } else {
                match (min, max) {
                    (0, Some(1)) => quote!(
                        if let ParsePairExpression::List(_, ref l) = #src {
                            l.first().is_some()
                        } else { #unreachable_exp }
                    ),
                    _ => quote!(
                        if let ParsePairExpression::List(_, ref l) = #src {
                            l.iter().len()
                        } else { #unreachable_exp }
                    ),
                }
            }
        }
        Expression::Literal(_) => return None,
        Expression::Sequence(c) => {
            let mut expressions = Vec::new();
            for (index, i) in c.iter().enumerate() {
                match i {
                    Expression::Sequence(_) => unreachable!(),
                    Expression::Choice(_) => todo!(),
                    Expression::Literal(_) => continue,
                    Expression::Negative(_) => continue,
                    Expression::Positive(_) => continue,
                    _ => {}
                }

                if let Some(line) = generate_unpack_expression(
                    i,
                    sort,
                    quote!(p[#index]),
                    ckr,
                    non_exhaustive.clone(),
                    sort_list,
                ) {
                    expressions.push(line)
                }
            }

            if expressions.is_empty() {
                return None;
            } else if let [ref expression] = expressions.as_slice() {
                quote!(
                    if let ParsePairExpression::List(_, ref l) = #src {
                        #expression
                    } else { #unreachable_exp }
                )
            } else {
                quote!(
                    if let ParsePairExpression::List(_, ref l) = #src {
                        (#(#expressions),*)
                    } else { #unreachable_exp }
                )
            }
        }
        a => unreachable!(
            "this expression should never be given to generate_unpack_expression: {:?}",
            a
        ),
    })
}

fn generate_unpack(
    sort: &str,
    constructor: TokenStream,
    expression: &Expression,
    no_layout: bool,
    ckr: &RecursionChecker,
    non_exhaustive: TokenStream,
    sort_list: &HashMap<&str, &Sort>,
) -> TokenStream {
    if no_layout {
        return quote!(
            return #constructor(info, pair.constructor_value.span().as_str().to_string());
        );
    }

    let unreachable_exp = quote!(unreachable!("expected different parse pair expression in pair to ast conversion of {}", #sort););

    match expression {
        a @ Expression::Sort(_) => {
            let nested = generate_unpack_expression(
                a,
                sort,
                quote!(pair.constructor_value),
                ckr,
                non_exhaustive.clone(),
                sort_list,
            );

            quote!(
                #constructor(info, #nested #non_exhaustive)
            )
        }
        Expression::Sequence(c) => {
            let mut expressions = Vec::new();
            for (index, i) in c.iter().enumerate() {
                match i {
                    Expression::Sequence(_) => unreachable!(),
                    Expression::Choice(_) => todo!(),
                    Expression::Literal(_) | Expression::Negative(_) | Expression::Positive(_) => {
                        continue;
                    }
                    _ => {}
                }

                if let Some(line) = generate_unpack_expression(
                    i,
                    sort,
                    quote!(l[#index]),
                    ckr,
                    non_exhaustive.clone(),
                    sort_list,
                ) {
                    expressions.push(line)
                }
            }

            if expressions.is_empty() {
                quote!(
                    #constructor(info #non_exhaustive)
                )
            } else {
                quote!(
                    if let ParsePairExpression::List(_, ref l) = pair.constructor_value {
                        #constructor(info, #(#expressions),* #non_exhaustive)
                    } else { #unreachable_exp }
                )
            }
        }
        a @ Expression::Repeat { .. }
        | a @ Expression::Delimited { .. }
        | a @ Expression::CharacterClass(_) => {
            if let Some(expression) = generate_unpack_expression(
                a,
                sort,
                quote!(pair.constructor_value),
                ckr,
                non_exhaustive.clone(),
                sort_list,
            ) {
                quote!(#constructor(info, #expression #non_exhaustive))
            } else {
                quote!(#constructor(info #non_exhaustive))
            }
        }
        Expression::Choice(_) => todo!(),
        Expression::Literal(_) => {
            quote!(#constructor(info #non_exhaustive))
        }
        Expression::Negative(_) => todo!(),
        Expression::Positive(_) => todo!(),
    }
}

pub fn flatten_sequences(syntax: Expression) -> Expression {
    match syntax {
        Expression::Sequence(s) => Expression::Sequence(
            s.into_iter()
                .flat_map(|i| match flatten_sequences(i) {
                    Expression::Sequence(s) => s,
                    a => vec![a],
                })
                .collect(),
        ),
        a => a,
    }
}

pub fn generate_from_pairs(
    syntax: &SyntaxFileAst,
    non_exhaustive: bool,
) -> Result<TokenStream, CodegenError> {
    let mut impls = Vec::new();

    let non_exhaustive = if non_exhaustive {
        quote!(, NonExhaustive)
    } else {
        TokenStream::new()
    };
    let sort_list = syntax.sorts.iter().map(|(k, v)| (k.as_str(), v)).collect::<HashMap<&str, &Sort>>();

    let arena = Default::default();
    let sorts_iterator = BreadthFirstAstIterator::new(syntax, &arena);

    for (sort, ckr) in sorts_iterator {
        if sort.annotations.contains(&Annotation::Hidden) {
            continue;
        }

        let sortname = format_ident!("{}", sanitize_identifier(&sort.name));
        let sortname_str = &sort.name;

        let unpack_body = if sort.constructors.len() == 1 {
            let constr = &sort.constructors[0];

            generate_unpack(
                &sort.name,
                quote!(Self),
                &flatten_sequences(constr.expression.clone()),
                constr.annotations.contains(&SingleString),
                ckr,
                non_exhaustive.clone(),
                &sort_list,
            )
        } else {
            let constructor_names_str = sort
                .constructors
                .iter()
                .map(|i| i.name.as_str())
                .collect_vec();

            let unpacks = sort
                .constructors
                .iter()
                .map(|constr| {
                    let name = format_ident!("{}", sanitize_identifier(&constr.name));

                    generate_unpack(
                        &sort.name,
                        quote!(
                            Self::#name
                        ),
                        &flatten_sequences(constr.expression.clone()),
                        constr.annotations.contains(&SingleString),
                        ckr,
                        non_exhaustive.clone(),
                        &sort_list,
                    )
                })
                .collect_vec();

            quote!(
                match pair.constructor_name {
                    #(
                        #constructor_names_str => #unpacks
                    ),*,
                    a => unreachable!("{}", a),
                }
            )
        };

        impls.push(quote!(
            impl<M: AstInfo> FromPairs<M> for #sortname<M> {
                fn from_pairs<G: GenerateAstInfo<Result = M>>(pair: &ParsePairSort, generator: &mut G) -> Self {
                    assert_eq!(pair.sort, #sortname_str);
                    let info = generator.generate(&pair);

                    #unpack_body
                }
            }
        ));
    }

    Ok(quote!(
        use super::prelude::*;

        #(#impls)*
    ))
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_from_pairs::flatten_sequences;
    use crate::parser::peg::parser_sugar_ast::Expression::{Literal, Sequence};

    #[test]
    fn test_flatten_sequences() {
        assert_eq!(
            flatten_sequences(Sequence(vec![Literal("a".to_string())])),
            Sequence(vec![Literal("a".to_string())])
        );
        assert_eq!(
            flatten_sequences(Sequence(vec![
                Literal("a".to_string()),
                Literal("b".to_string()),
            ])),
            Sequence(vec![Literal("a".to_string()), Literal("b".to_string()),])
        );
        assert_eq!(
            flatten_sequences(Sequence(vec![
                Sequence(vec![Literal("a".to_string()), Literal("b".to_string()),]),
                Sequence(vec![
                    Literal("c".to_string()),
                    Sequence(vec![Literal("d".to_string()), Literal("e".to_string()),])
                ])
            ])),
            Sequence(vec![
                Literal("a".to_string()),
                Literal("b".to_string()),
                Literal("c".to_string()),
                Literal("d".to_string()),
                Literal("e".to_string()),
            ])
        );
    }
}

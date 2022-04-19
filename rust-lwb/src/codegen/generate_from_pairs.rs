use crate::codegen::error::CodegenError;
use crate::codegen::{sanitize_identifier, FormattingFile};
use crate::parser::peg::parser_sugar_ast::Annotation::SingleString;
use crate::parser::peg::parser_sugar_ast::{Expression, SyntaxFileAst};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::io::Write;

fn generate_unpack_expression(
    expression: &Expression,
    sort: &str,
    src: TokenStream,
) -> Option<TokenStream> {
    let unreachable_exp = quote!(unreachable!("expected different parse pair expression in pair to ast conversion of {}", #sort););

    Some(match expression {
        Expression::Sort(name) => {
            let name = format_ident!("{}", sanitize_identifier(name));

            quote!(
                if let ParsePairExpression::Sort(_, ref s) = #src {
                    Box::new(#name::from_pairs(s, generator))
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
            if let Some(ue) = generate_unpack_expression(e, sort, quote!(x)) {
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

                if let Some(line) = generate_unpack_expression(i, sort, quote!(p[#index])) {
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
) -> TokenStream {
    if no_layout {
        return quote!(
            return #constructor(info, pair.constructor_value.span().as_str().to_string());
        );
    }

    let unreachable_exp = quote!(unreachable!("expected different parse pair expression in pair to ast conversion of {}", #sort););

    match expression {
        a @ Expression::Sort(_) => {
            let nested = generate_unpack_expression(a, sort, quote!(pair.constructor_value));

            quote!(
                #constructor(info, #nested)
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

                if let Some(line) = generate_unpack_expression(i, sort, quote!(l[#index])) {
                    expressions.push(line)
                }
            }

            if expressions.is_empty() {
                quote!(
                    #constructor(info)
                )
            } else {
                quote!(
                    if let ParsePairExpression::List(_, ref l) = pair.constructor_value {
                        #constructor(info, #(#expressions),*)
                    } else { #unreachable_exp }
                )
            }
        }
        a @ Expression::Repeat { .. }
        | a @ Expression::Delimited { .. }
        | a @ Expression::CharacterClass(_) => {
            if let Some(expression) =
                generate_unpack_expression(a, sort, quote!(pair.constructor_value))
            {
                quote!(#constructor(info, #expression))
            } else {
                quote!(#constructor(info))
            }
        }
        Expression::Choice(_) => todo!(),
        Expression::Literal(_) => {
            quote!(#constructor(info))
        }
        Expression::Negative(_) => todo!(),
        Expression::Positive(_) => todo!(),
    }
}

pub fn write_from_pairs(
    file: &mut FormattingFile,
    syntax: &SyntaxFileAst,
) -> Result<(), CodegenError> {
    let mut impls = Vec::new();

    for sort in &syntax.sorts {
        let sortname = format_ident!("{}", sanitize_identifier(&sort.name));
        let sortname_str = &sort.name;

        let unpack_body = if sort.constructors.len() == 1 {
            let constr = &sort.constructors[0];

            generate_unpack(
                &sort.name,
                quote!(Self),
                &constr.expression,
                constr.annotations.contains(&SingleString),
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
                        &constr.expression,
                        constr.annotations.contains(&SingleString),
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

    write!(
        file,
        "{}",
        quote!(
            use super::prelude::*;

            #(#impls)*
        )
    )?;

    Ok(())
}

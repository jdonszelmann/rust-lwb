use crate::codegen::check_recursive::{BreadthFirstAstIterator, RecursionChecker};
use crate::codegen::error::CodegenError;
use crate::codegen::generate_misc::generate_serde_attrs;
use crate::codegen::sanitize_identifier;
use crate::parser::peg::parser_sugar_ast::Annotation::SingleString;
use crate::parser::peg::parser_sugar_ast::{Annotation, Expression, Sort, SyntaxFileAst};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;

pub fn convert_docs(docs: Option<&String>) -> Vec<TokenStream> {
    docs.cloned()
        .unwrap_or_default()
        .lines()
        .map(|i| quote!(#[doc=#i]))
        .collect_vec()
}

pub fn generate_structs(
    syntax: &SyntaxFileAst,
    derives: &[&str],
    non_exhaustive: bool,
) -> Result<TokenStream, CodegenError> {
    let mut items = Vec::new();

    let serde_attrs = generate_serde_attrs(derives);

    let (
        non_exhaustive_struct_field,
        non_exhaustive_enum_field,
        non_exhaustive_attr,
        non_exhaustive_enum_variant,
    ) = if non_exhaustive {
        (
            quote!(, #[doc(hidden)] pub NonExhaustive),
            quote!(, #[doc(hidden)] NonExhaustive),
            quote!(#[non_exhaustive]),
            quote!(,
                #[doc(hidden)]
                __NonExhaustive(NonExhaustive),
            ),
        )
    } else {
        (
            TokenStream::new(),
            TokenStream::new(),
            TokenStream::new(),
            TokenStream::new(),
        )
    };

    let derives = derives.iter().map(|i| format_ident!("{}", i)).collect_vec();

    let sort_list = syntax
        .sorts
        .iter()
        .map(|(k, v)| (k.as_str(), v))
        .collect::<HashMap<&str, &Sort>>();

    let arena = Default::default();
    let sorts_iterator = BreadthFirstAstIterator::new(syntax, &arena);

    for (rule, ckr) in sorts_iterator {
        if rule.annotations.contains(&Annotation::Hidden) {
            continue;
        }

        if rule.constructors.len() == 1 {
            let name = format_ident!("{}", sanitize_identifier(&rule.name));

            let doc = convert_docs(rule.documentation.as_ref());
            let constr = &rule.constructors[0];

            if constr
                .annotations
                .iter()
                .any(|i| matches!(i, &Annotation::Error(_)))
            {
                // continues outer loop (over sorts)
                continue;
            }

            if constr.annotations.contains(&SingleString) {
                items.push(quote!(
                    #(#doc)*
                    #[derive(#(#derives),*)]
                    #serde_attrs
                    pub struct #name<M>(pub M, pub std::string::String);
                ));
            } else {
                let c = generate_constructor_type(&constr.expression, ckr, &sort_list);
                let fields = c.flatten().map(|i| quote!(pub #i)).collect_vec();

                items.push(quote!(
                    #(#doc)*
                    #[derive(#(#derives),*)]
                    #non_exhaustive_attr
                    #serde_attrs
                    pub struct #name<M>(
                        pub M,
                        #(#fields),*
                        #non_exhaustive_struct_field
                    );
                ));
            }
        } else {
            let name = format_ident!("{}", sanitize_identifier(&rule.name));
            let doc = convert_docs(rule.documentation.as_ref());

            let mut variants = Vec::new();

            for constr in &rule.constructors {
                if constr
                    .annotations
                    .iter()
                    .any(|i| matches!(i, &Annotation::Error(_)))
                {
                    continue;
                }

                let name = format_ident!("{}", sanitize_identifier(&constr.name));
                let doc = convert_docs(constr.documentation.as_ref());

                if constr.annotations.contains(&SingleString) {
                    variants.push(quote!(
                        #(#doc)*
                        #name(M, std::string::String #non_exhaustive_enum_field)
                    ));
                } else {
                    let c = generate_constructor_type(&constr.expression, ckr, &sort_list);
                    let fields = c.flatten().collect_vec();

                    if fields.is_empty() {
                        variants.push(quote!(
                            #(#doc)*
                            #name(M #non_exhaustive_enum_field)
                        ))
                    } else {
                        variants.push(quote!(
                            #(#doc)*
                            #name(M, #(#fields),* #non_exhaustive_enum_field)
                        ))
                    }
                };
            }

            items.push(quote!(
                #(#doc)*
                #[derive(#(#derives),*)]
                #non_exhaustive_attr
                #serde_attrs
                pub enum #name<M> {
                    #(#variants),*
                    #non_exhaustive_enum_variant
                }
            ));
        }
    }

    let start_sort_identifier = format_ident!("{}", sanitize_identifier(&syntax.starting_sort));

    Ok(quote!(
        use super::prelude::*;

        #(
            #items
        )*

        pub type AST_ROOT<M> = #start_sort_identifier<M>;
    ))
}

#[derive(Eq, PartialEq)]
enum Tree<T> {
    Leaf(T),
    Node(Vec<Tree<T>>),
    Empty,
}

impl<T> Tree<T> {
    pub fn flatten(&self) -> TreeIterator<T> {
        TreeIterator { todo: vec![self] }
    }
}

struct TreeIterator<'a, T> {
    todo: Vec<&'a Tree<T>>,
}

impl<'a, T> Iterator for TreeIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.todo.pop() {
                None => return None,
                Some(Tree::Empty) => {}
                Some(Tree::Leaf(t)) => return Some(t),
                Some(Tree::Node(n)) => {
                    self.todo.extend(n.iter().rev());
                }
            }
        }
    }
}

fn generate_constructor_type(
    constructor: &Expression,
    ckr: &RecursionChecker,
    sort_list: &HashMap<&str, &Sort>,
) -> Tree<TokenStream> {
    match constructor {
        Expression::Sort(sort) => {
            if sort_list
                .get(sort.as_str())
                .map(|i| i.annotations.contains(&Annotation::Hidden))
                .unwrap_or_default()
            {
                return Tree::Empty;
            }

            let name = format_ident!("{}", sanitize_identifier(sort));
            Tree::Leaf(ckr.maybe_box_type(
                sort,
                quote!(
                    #name<M>
                ),
            ))
        }
        Expression::Sequence(cons) => {
            let mut parts: Vec<Tree<_>> = cons
                .iter()
                .filter_map(|con| match generate_constructor_type(con, ckr, sort_list) {
                    Tree::Empty => None,
                    x => Some(x),
                })
                .collect_vec();

            if parts.is_empty() {
                Tree::Empty
            } else if parts.len() == 1 {
                parts.pop().unwrap()
            } else {
                Tree::Node(parts)
            }
        }
        Expression::Repeat { e, min, max } | Expression::Delimited { e, min, max, .. } => {
            let subtype = generate_constructor_type(e.as_ref(), ckr, sort_list);
            let flattened_subtype = subtype.flatten().collect_vec();

            match (min, max) {
                (0, Some(1)) if matches!(subtype, Tree::Empty) => Tree::Leaf(quote!(bool)),
                (0, Some(1)) if flattened_subtype.len() == 1 => {
                    let elem = flattened_subtype[0];

                    Tree::Leaf(quote!(Option<#elem>))
                }
                (0, Some(1)) => Tree::Leaf(quote!(Option<
                    (#(#flattened_subtype),*)
                >)),
                _ if matches!(subtype, Tree::Empty) => Tree::Leaf(quote!(usize)),
                _ if flattened_subtype.len() == 1 => {
                    let elem = flattened_subtype[0];

                    Tree::Leaf(quote!(Vec<#elem>))
                }
                _ => Tree::Leaf(quote!(Vec<(
                    #(#flattened_subtype>)*,
                )>)),
            }
        }
        Expression::Choice(_) => panic!(), //TODO how to represent choice?
        Expression::CharacterClass(_) => Tree::Leaf(quote!(std::string::String)),
        Expression::Negative(_) => Tree::Empty,
        Expression::Positive(_) => Tree::Empty,
        Expression::Literal(_) => Tree::Empty,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_flatten_tree() {
        use crate::codegen::generate_structs::Tree::*;

        let t = Node(vec![Empty, Leaf(3), Leaf(4)]);
        let mut ti = t.flatten();

        assert_eq!(ti.next(), Some(&3));
        assert_eq!(ti.next(), Some(&4));
    }

    #[test]
    fn test_flatten_tree_nested() {
        use crate::codegen::generate_structs::Tree::*;

        let t = Node(vec![Empty, Node(vec![Leaf(3), Leaf(4)]), Leaf(5)]);
        let mut ti = t.flatten();

        assert_eq!(ti.next(), Some(&3));
        assert_eq!(ti.next(), Some(&4));
        assert_eq!(ti.next(), Some(&5));
    }
}

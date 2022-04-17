use crate::codegen::error::CodegenError;
use crate::codegen::sanitize_identifier;
use crate::parser::peg::parser_sugar_ast::Annotation::SingleString;
use crate::parser::peg::parser_sugar_ast::{Expression, SyntaxFileAst};
use codegen::Scope;
use itertools::Itertools;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;

pub fn write_ast(
    file: &mut File,
    syntax: &SyntaxFileAst,
    derives: &[&str],
) -> Result<(), CodegenError> {
    let mut scope = Scope::new();
    scope.import("super::prelude", "*");

    for rule in &syntax.sorts {
        if rule.constructors.len() == 1 {
            let structt = scope.new_struct(&sanitize_identifier(&rule.name));
            structt.vis("pub");

            for i in derives {
                structt.derive(i);
            }

            structt.generic("M : AstInfo");
            let constr = &rule.constructors[0];
            structt.tuple_field("pub M");

            let typ = if constr.annotations.contains(&SingleString) {
                "pub String".to_string()
            } else {
                match generate_constructor_type(&constr.expression) {
                    Tree::Leaf(s) => format!("pub {}", s),
                    Tree::Node(trees) => {
                        let mut buf = String::new();
                        for t in trees {
                            buf.push_str(&format!("pub {}, ", t));
                        }
                        buf
                    }
                    Tree::Empty => "()".to_string(),
                }
            };

            let typ = if typ.starts_with('(') {
                &typ[1..typ.len() - 1]
            } else {
                &typ
            };
            structt.tuple_field(typ);
        } else {
            let enumm = scope.new_enum(&sanitize_identifier(&rule.name));
            enumm.vis("pub");

            for i in derives {
                enumm.derive(i);
            }

            enumm.generic("M : AstInfo");
            for constr in &rule.constructors {
                let variant = enumm.new_variant(&sanitize_identifier(&constr.name));
                variant.tuple("M");

                let typ = if constr.annotations.contains(&SingleString) {
                    "String".to_string()
                } else {
                    generate_constructor_type(&constr.expression).to_string()
                };

                let typ = if typ.starts_with('(') {
                    &typ[1..typ.len() - 1]
                } else {
                    &typ
                };
                variant.tuple(typ);
            }
        }
    }

    scope.raw(&format!(
        "pub type AST_ROOT<M> = {}<M>;",
        sanitize_identifier(&syntax.starting_sort)
    ));

    write!(file, "{}", scope.to_string())?;

    Ok(())
}

#[derive(Eq, PartialEq)]
enum Tree<T> {
    Leaf(T),
    Node(Vec<Tree<T>>),
    Empty,
}

impl Display for Tree<String> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Tree::Leaf(s) => write!(f, "{}", s),
            Tree::Node(trees) => {
                for t in trees {
                    write!(f, "{}, ", t)?;
                }
                Ok(())
            }
            Tree::Empty => write!(f, "()"),
        }
    }
}

fn generate_constructor_type(constructor: &Expression) -> Tree<String> {
    match constructor {
        Expression::Sort(sort) => Tree::Leaf(format!("Box<{}<M>>", sanitize_identifier(sort))),
        Expression::Sequence(cons) => {
            let mut parts: Vec<Tree<String>> = cons
                .iter()
                .filter_map(|con| match generate_constructor_type(con) {
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
            let subtype = generate_constructor_type(e.as_ref());

            match (min, max) {
                (0, Some(1)) if subtype == Tree::Empty => Tree::Leaf("bool".to_string()),
                (0, Some(1)) => Tree::Leaf(format!("Option<{}>", subtype)),
                _ if subtype == Tree::Empty => Tree::Leaf("usize".to_string()),
                _ => Tree::Leaf(format!("Vec<{}>", subtype)),
            }
        }
        Expression::Choice(_) => panic!(), //TODO how to represent choice?
        Expression::CharacterClass(_) => Tree::Leaf("String".to_string()),
        Expression::Negative(_) => Tree::Empty,
        Expression::Positive(_) => Tree::Empty,
        Expression::Literal(_) => Tree::Empty,
    }
}

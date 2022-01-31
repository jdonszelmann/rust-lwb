use crate::parser::bootstrap::ast::{Expression, SyntaxFileAst};
use codegen::{Function, Scope};
use convert_case::{Case, Casing};
use std::ops::Deref;

pub mod manager;

fn sanitize_identifier(id: &str) -> String {
    id.to_case(Case::UpperCamel)
}

fn generate_unpack_expression(expression: &Expression, sort: &str, src: &str) -> String {
    match expression {
        Expression::Sort(name) => format!(
            r#"if let ParsePairExpression::Sort(_, ref s) = {src} {{
            Box::new({}::from_pairs(s, generator))
        }} else {{
            panic!("expected empty parse pair expression in pair to ast conversion of {sort}")
        }}"#,
            sanitize_identifier(name)
        ),
        Expression::CharacterClass(_) => format!(
            r#"if let ParsePairExpression::Empty(ref span) = {src} {{
    span.as_str().to_string()
}} else {{
    panic!("expected empty parse pair expression in pair to ast conversion of {sort}")
}}"#
        ),
        Expression::Repeat { min, max, c } => {
            match (min, max) {
                (0, Some(1)) => {
                    // option
                    format!(
                        r#"if let ParsePairExpression::List(_, ref l) = {src} {{
    l.first().map(|x| {{ {} }})
}} else {{
    panic!("expected empty parse pair expression in pair to ast conversion of {sort}")
}}
                    "#,
                        generate_unpack_expression(c, sort, "x")
                    )
                }
                _ => {
                    // vec
                    format!(
                        r#"if let ParsePairExpression::List(_, ref l) = {src} {{
    l.iter().map(|x| {{ {} }}).collect()
}} else {{
    panic!("expected empty parse pair expression in pair to ast conversion of {sort}")
}}
                    "#,
                        generate_unpack_expression(c, sort, "x")
                    )
                }
            }
        }
        Expression::Literal(_) => "()".to_string(),
        a => unreachable!("{:?}", a),
    }
}

fn generate_unpack(f: &mut Function, sort: &str, constructor: &str, expression: &Expression) {
    match expression {
        a @ Expression::Sort(_) => {
            f.line(format!(
                "Self::{constructor}(info, {})",
                generate_unpack_expression(a, sort, "pair.constructor_value")
            ));
        }
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

                let line = generate_unpack_expression(i, sort, &format!("p[{index}]"));
                expressions.push(line)
            }

            f.line(format!(r#"if let ParsePairExpression::List(_, ref p) = pair.constructor_value {{
                    Self::{constructor}(info, {})
                }} else {{
                    panic!("expected empty parse pair expression in pair to ast conversion of {sort}")
                }}"#
            , expressions.join(",")));
        }
        a @ Expression::Repeat { .. } => {
            f.line(format!(
                "Self::{constructor}(info, {})",
                generate_unpack_expression(a, sort, "pair.constructor_value")
            ));
        }
        a @ Expression::CharacterClass(_) => {
            f.line(format!(
                "Self::{constructor}(info, {})",
                generate_unpack_expression(a, sort, "pair.constructor_value")
            ));
        }

        Expression::Choice(_) => todo!(),

        Expression::Literal(_) => {
            f.line(format!("Self::{constructor}(info)"));
        }
        Expression::Negative(_) => todo!(),
        Expression::Positive(_) => todo!(),
    }
}

pub fn generate_language(syntax: SyntaxFileAst) -> String {
    let mut scope = Scope::new();

    scope.import("rust_lwb::codegen_prelude", "*");

    for rule in &syntax.sorts {
        let enumm = scope.new_enum(&sanitize_identifier(&rule.name));
        enumm.generic("M : AstInfo");
        for constr in &rule.constructors {
            let variant = enumm.new_variant(&sanitize_identifier(&constr.name));
            variant.tuple("M");
            let typ =
                generate_constructor_type(&constr.constructor).unwrap_or_else(|| "()".to_string());
            let typ = if typ.starts_with('(') {
                &typ[1..typ.len() - 1]
            } else {
                &typ
            };
            variant.tuple(typ);
        }
    }

    // TODO: put these definition in a different file
    for rule in &syntax.sorts {
        let mut f = Function::new("from_pairs");
        f.generic("G: GenerateAstInfo<Result = M>");
        f.arg("pair", "&ParsePairSort");
        f.arg("generator", "&mut G");
        f.ret("Self");
        f.line(format!(r#"assert_eq!(pair.sort, "{}");"#, rule.name));
        f.line("let info = generator.generate(&pair);");
        f.line("match pair.constructor_name {");

        for constructor in &rule.constructors {
            f.line(format!(r#""{}" => {{"#, constructor.name));
            generate_unpack(
                &mut f,
                &rule.name,
                &sanitize_identifier(&constructor.name),
                &constructor.constructor,
            );
            f.line("}");
        }

        f.line(r#"a => unreachable!("{}", a)"#);
        f.line("}");

        scope
            .new_impl(&format!("{}<M>", sanitize_identifier(&rule.name)))
            .impl_trait("FromPairs<M>")
            .generic("M: AstInfo")
            .push_fn(f);
    }

    format!(
        "
#![allow(unused)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::all)]
// |==========================================================|
// | WARNING: THIS FILE IS AUTOMATICALLY GENERATED.           |
// | CHANGES TO IT WILL BE DELETED WHEN REGENERATED.          |
// | IN GENERAL, THIS FILE SHOULD NOT BE MODIFIED IN ANY WAY. |
// |==========================================================|


{}",
        scope.to_string()
    )
}

fn generate_constructor_type(constructor: &Expression) -> Option<String> {
    match constructor {
        Expression::Sort(sort) => Some(String::from_iter([
            "Box<",
            &sanitize_identifier(sort),
            "<M>>",
        ])),
        Expression::Sequence(cons) => {
            let mut s = String::new();
            s.push('(');
            for con in cons {
                if let Some(con_type) = generate_constructor_type(con) {
                    s.push_str(&con_type);
                    s.push(',');
                }
            }
            s.push(')');
            if s.len() > 2 {
                Some(s)
            } else {
                None
            }
        }
        Expression::Repeat { c, min, max } => {
            let subtype = if let Expression::Literal(..) = c.deref() {
                "()".to_string()
            } else {
                generate_constructor_type(c.as_ref())?
            };

            match (min, max) {
                (0, Some(1)) => Some(String::from_iter(["Option<", &subtype, ">"])),
                _ => Some(String::from_iter(["Vec<", &subtype, ">"])),
            }
        }
        Expression::Choice(_) => None, //TODO how to represent choice?
        Expression::CharacterClass(_) => Some("String".to_string()),
        Expression::Negative(_) => None,
        Expression::Positive(_) => None,
        Expression::Literal(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_language;
    use crate::parser::bootstrap::ast::{Constructor, Expression, Sort, SyntaxFileAst};
    use crate::sources::character_class::CharacterClass;

    #[test]
    pub fn run_example() {
        let ast = SyntaxFileAst {
            sorts: vec![Sort {
                name: "AS".to_string(),
                constructors: vec![
                    Constructor {
                        name: "More".to_string(),
                        constructor: Expression::Sequence(vec![
                            Expression::Literal("a".to_string()),
                            Expression::Sort("AS".to_string()),
                        ]),
                        annotations: vec![],
                    },
                    Constructor {
                        name: "NoMore".to_string(),
                        constructor: Expression::Sequence(vec![]),
                        annotations: vec![],
                    },
                ],
            }],
            starting_sort: "A".to_string(),
            layout: CharacterClass::Nothing,
        };
        generate_language(ast);
    }
}

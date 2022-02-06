use crate::codegen::error::CodegenError;
use crate::codegen::sanitize_identifier;
use crate::parser::bootstrap::ast::Annotation::SingleString;
use crate::parser::bootstrap::ast::{Expression, SyntaxFileAst};
use codegen::Scope;
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
                generate_constructor_type(&constr.expression).unwrap_or_else(|| "()".to_string())
            };

            let typ = if typ.starts_with('(') {
                &typ[1..typ.len() - 1]
            } else {
                &typ
            };
            variant.tuple(typ);
        }
    }

    scope.raw(&format!(
        "pub type AST_ROOT<M> = {}<M>;",
        sanitize_identifier(&syntax.starting_sort)
    ));

    write!(file, "{}", scope.to_string())?;

    Ok(())
}

fn generate_constructor_type(constructor: &Expression) -> Option<String> {
    match constructor {
        Expression::Sort(sort) => Some(format!("Box<{}<M>>", sanitize_identifier(sort))),
        Expression::Sequence(cons) => {
            let mut parts = Vec::new();

            for con in cons {
                if let Some(con_type) = generate_constructor_type(con) {
                    parts.push(con_type);
                }
            }

            if parts.is_empty() {
                None
            } else if parts.len() == 1 {
                Some(parts.pop().unwrap())
            } else {
                Some(format!("({})", parts.join(",")))
            }
        }
        Expression::Repeat { c, min, max } => {
            let subtype = if let Expression::Literal(..) = &**c {
                "()".to_string()
            } else {
                generate_constructor_type(c.as_ref())?
            };

            match (min, max) {
                (0, Some(1)) if subtype == "()" => Some("bool".to_string()),
                (0, Some(1)) => Some(String::from_iter(["Option<", &subtype, ">"])),
                _ if subtype == "()" => Some("usize".to_string()),
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

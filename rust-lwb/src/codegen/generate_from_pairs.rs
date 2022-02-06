use crate::codegen::error::CodegenError;
use crate::codegen::generate_trait_impls::{build_function, build_trait_impl};
use crate::codegen::sanitize_identifier;
use crate::parser::bootstrap::ast::Annotation::SingleString;
use crate::parser::bootstrap::ast::{Expression, SyntaxFileAst};
use codegen::{Function, Scope};
use std::fs::File;
use std::io::Write;

fn generate_unpack_expression(expression: &Expression, sort: &str, src: &str) -> String {
    match expression {
        Expression::Sort(name) => format!(
            r#"
if let ParsePairExpression::Sort(_, ref s) = {src} {{
    Box::new({}::from_pairs(s, generator))
}} else {{
    panic!("expected empty parse pair expression in pair to ast conversion of {sort}")
}}
            "#,
            sanitize_identifier(name)
        ),
        Expression::CharacterClass(_) => format!(
            r#"
if let ParsePairExpression::Empty(ref span) = {src} {{
    span.as_str().to_string()
}} else {{
    panic!("expected empty parse pair expression in pair to ast conversion of {sort}")
}}
        "#
        ),
        Expression::Repeat { min, max, c } => {
            let ue = generate_unpack_expression(c, sort, "x");

            match (min, max) {
                (0, Some(1)) if ue == "()" => {
                    format!(
                        r#"
if let ParsePairExpression::List(_, ref l) = {src} {{
    l.first().is_some()
}} else {{
    panic!("expected empty parse pair expression in pair to ast conversion of {sort}")
}}
                    "#,
                    )
                }
                (0, Some(1)) => {
                    // option
                    format!(
                        r#"
if let ParsePairExpression::List(_, ref l) = {src} {{
    l.first().map(|x| {{ {} }})
}} else {{
    panic!("expected empty parse pair expression in pair to ast conversion of {sort}")
}}
                    "#,
                        ue
                    )
                }
                _ if ue == "()" => {
                    format!(
                        r#"
if let ParsePairExpression::List(_, ref l) = {src} {{
    l.iter().len()
}} else {{
    panic!("expected empty parse pair expression in pair to ast conversion of {sort}")
}}
                    "#,
                    )
                }
                _ => {
                    // vec
                    format!(
                        r#"
if let ParsePairExpression::List(_, ref l) = {src} {{
    l.iter().map(|x| {{ {} }}).collect()
}} else {{
    panic!("expected empty parse pair expression in pair to ast conversion of {sort}")
}}
                    "#,
                        ue
                    )
                }
            }
        }
        Expression::Literal(_) => "()".to_string(),
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

            if expressions.is_empty() {
                todo!()
            } else if expressions.len() == 1 {
                let line = format!(
                    r#"
if let ParsePairExpression::List(_, ref p) = {src} {{
    {}
}} else {{
    panic!("expected empty parse pair expression in pair to ast conversion of {sort}")
}}
                    "#,
                    expressions.pop().unwrap()
                );

                line
            } else {
                let line = format!(
                    r#"if let ParsePairExpression::List(_, ref p) = {src} {{
    ({})
}} else {{
    panic!("expected empty parse pair expression in pair to ast conversion of {sort}")
}}"#,
                    expressions.join(",")
                );

                line
            }
        }
        a => unreachable!("{:?}", a),
    }
}

fn generate_unpack(
    f: &mut Function,
    sort: &str,
    constructor: &str,
    expression: &Expression,
    no_layout: bool,
) {
    // if its a no-layout constructor then just return the contents as a string
    if no_layout {
        f.line(format!(
            "Self::{constructor}(info, pair.constructor_value.span().as_str().to_string())"
        ));
        return;
    }

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
                    Expression::Literal(_) | Expression::Negative(_) | Expression::Positive(_) => {
                        continue
                    }
                    _ => {}
                }

                let line = generate_unpack_expression(i, sort, &format!("p[{index}]"));
                expressions.push(line)
            }

            f.line(format!(
                r#"
if let ParsePairExpression::List(_, ref p) = pair.constructor_value {{
    Self::{constructor}(info, {})
}} else {{
    panic!("expected empty parse pair expression in pair to ast conversion of {sort}")
}}
            "#,
                expressions.join(",")
            ));
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

pub fn write_from_pairs(file: &mut File, syntax: &SyntaxFileAst) -> Result<(), CodegenError> {
    let mut scope = Scope::new();
    scope.import("super::prelude", "*");

    for sort in &syntax.sorts {
        let sortname = sanitize_identifier(&sort.name);
        build_trait_impl(
            &mut scope,
            "FromPairs<M>",
            format!("{sortname}<M>"),
            vec![build_function("from_pairs", |f| {
                f.generic("G: GenerateAstInfo<Result = M>");
                f.arg("pair", "&ParsePairSort");
                f.arg("generator", "&mut G");
                f.ret("Self");

                f.line(format!(r#"assert_eq!(pair.sort, "{}");"#, sort.name));
                f.line("let info = generator.generate(&pair);");

                f.line("match pair.constructor_name {");

                for constructor in &sort.constructors {
                    f.line(format!(r#""{}" => {{"#, constructor.name));
                    generate_unpack(
                        f,
                        &sort.name,
                        &sanitize_identifier(&constructor.name),
                        &constructor.expression,
                        constructor.annotations.contains(&SingleString),
                    );
                    f.line("}");
                }

                f.line(r#"a => unreachable!("{}", a)"#);
                f.line("}");
            })],
        )
        .generic("M: AstInfo");
    }

    write!(file, "{}", scope.to_string())?;

    Ok(())
}

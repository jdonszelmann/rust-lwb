use codegen::Scope;
use crate::parser::ast::AstInfo;
use crate::parser::syntax_file::ast::{Constructor, SyntaxFileAst};

pub fn generate_language(syntax: SyntaxFileAst) {
    let mut scope = Scope::new();

    for rule in &syntax.sorts {
        let enumm = scope.new_enum(&rule.name);
        enumm.generic("M : AstInfo");
        for constr in &rule.constructors {
            let variant = enumm.new_variant(&constr.0);
            variant.tuple("M");
            let typ = generate_constructor_type(&constr.1).unwrap_or("()".to_string());
            let typ = if typ.starts_with("(") { &typ[1..typ.len() - 1] } else { &typ };
            variant.tuple(typ);
        }
    }
    println!("{}", scope.to_string())
}

fn generate_constructor_type(constructor: &Constructor) -> Option<String> {
    match constructor {
        Constructor::Sort(sort) => Some(String::from_iter(["Box<", sort, "<M>>"])),
        Constructor::Sequence(cons) => {
            let mut s = String::new();
            s.push_str("(");
            for con in cons {
                if let Some(con_type) = generate_constructor_type(con) {
                    s.push_str(&con_type);
                    s.push_str(",");
                }
            }
            s.push_str(")");
            if s.len() > 2 {
                Some(s)
            } else {
                None
            }
        }
        Constructor::Repeat { c, min, max } => {
            let subtype = generate_constructor_type(c.as_ref())?;
            match (min, max) {
                (0, Some(1)) => Some(String::from_iter(["Option<", &subtype, ">"])),
                _ => Some(String::from_iter(["Vec<", &subtype, ">"]))
            }
        },
        Constructor::Choice(_) => None, //TODO how to represent choice?
        Constructor::CharacterClass(_) => None,
        Constructor::Negative(_) => None,
        Constructor::Positive(_) => None,
        Constructor::Literal(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::codegen::codegen::generate_language;
    use crate::parser::syntax_file::ast::{Constructor, Sort, SyntaxFileAst};
    use crate::parser::syntax_file::character_class::CharacterClass;

    #[test]
    pub fn run_example() {
        let ast = SyntaxFileAst {
            sorts: vec![
                Sort {
                    name: "AS".to_string(),
                    constructors: vec![
                        ("More".to_string(), Constructor::Sequence(vec![
                            Constructor::Literal("a".to_string()),
                            Constructor::Sort("AS".to_string()),
                        ])),
                        ("NoMore".to_string(), Constructor::Sequence(vec![]))
                    ],
                    annotations: vec![]
                }
            ],
            starting_sort: "A".to_string(),
            layout: CharacterClass::Nothing
        };
        generate_language(ast);
    }
}
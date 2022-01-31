use codegen::Scope;
use crate::parser::bootstrap::ast::{Constructor, SyntaxFileAst};

pub fn generate_language(syntax: SyntaxFileAst) -> String {
    let mut scope = Scope::new();


    scope.import("rust_lwb_parser::prelude", "*");

    for rule in &syntax.sorts {
        let enumm = scope.new_enum(&rule.name);
        enumm.generic("M : AstInfo");
        for constr in &rule.constructors {
            let variant = enumm.new_variant(&constr.name);
            variant.tuple("M");
            let typ = generate_constructor_type(&constr.constructor).unwrap_or("()".to_string());
            let typ = if typ.starts_with("(") { &typ[1..typ.len() - 1] } else { &typ };
            variant.tuple(typ);
        }
    }

    scope.to_string()
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
    use crate::parser::bootstrap::ast::{Constructor, Sort, SyntaxFileAst, TopLevelConstructor};
    use crate::sources::character_class::CharacterClass;

    #[test]
    pub fn run_example() {
        let ast = SyntaxFileAst {
            sorts: vec![
                Sort {
                    name: "AS".to_string(),
                    constructors: vec![
                        TopLevelConstructor{ name: "More".to_string(), constructor: Constructor::Sequence(vec![
                            Constructor::Literal("a".to_string()),
                            Constructor::Sort("AS".to_string()),
                        ]),
                            annotations: vec![]
                        },
                        TopLevelConstructor{ name: "NoMore".to_string(), constructor: Constructor::Sequence(vec![]), annotations: vec![] }
                    ],
                }
            ],
            starting_sort: "A".to_string(),
            layout: CharacterClass::Nothing
        };
        generate_language(ast);
    }
}
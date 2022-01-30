use codegen::Scope;
use crate::parser::syntax_file::ast::{Constructor, SyntaxFileAst};

pub fn generate_language(syntax: SyntaxFileAst) {
    let mut scope = Scope::new();

    for rule in &syntax.sorts {
        let enumm = scope.new_enum(&rule.name);
        for constr in &rule.constructors {
            let variant = enumm.new_variant(&constr.0);
            variant.tuple("Span");
            variant.tuple(&generate_constructor_type(&constr.1));
        }
    }
    println!("{}", scope.to_string())
}

fn generate_constructor_type(constructor: &Constructor) -> String {
    match constructor {
        Constructor::Sort(sort) => String::from_iter(["Box<", sort, ">"]),
        Constructor::Literal(_) => "()".to_string(),
        Constructor::Sequence(cons) => {
            let mut s = String::new();
            s.push_str("(");
            for con in cons {
                s.push_str(&generate_constructor_type(con));
                s.push_str(",");
            }
            s.push_str(")");
            s
        }
        Constructor::Repeat { .. } => "".to_string(),
        Constructor::CharacterClass(_) => "".to_string(),
        Constructor::Choice(_) => "".to_string(),
        Constructor::Negative(_) => "".to_string(),
        Constructor::Positive(_) => "".to_string(),
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
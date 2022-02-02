use crate::codegen_prelude::AstInfo;
use crate::parser::syntax_file::ast;
use crate::parser::bootstrap::ast::{Constructor, Sort, SyntaxFileAst};
use crate::parser::syntax_file::AST::{Identifier, Meta, SortOrMeta};
use crate::sources::character_class::CharacterClass;

pub fn convert_syntax_file_ast<M: AstInfo>(inp: ast::AST_ROOT<M>) -> SyntaxFileAst {
    match inp {
        ast::Program::Program(_, sort_or_metas) => {
            let mut layout = CharacterClass::Nothing;
            let mut sorts = Vec::new();

            for i in sort_or_metas {
                match *i {
                    SortOrMeta::Meta(_, m) => {
                        match *m {
                            Meta::Layout(_, l) => {
                                layout = layout.combine(convert_character_class(*l))
                            }
                            Meta::Start(_, name) => {

                            }
                        }
                    }
                    SortOrMeta::Sort(_, sort) => {
                        sorts.push(convert_sort(*sort))
                    }
                }
            }

            SyntaxFileAst {
                sorts,
                starting_sort: "".to_string(),
                layout: CharacterClass::Nothing
            }
        }
    }
}

fn convert_identifier<M: AstInfo>(inp: ast::Identifier<M>) -> String {
    match inp {
        Identifier::Identifier(_, name) => {
            name
        }
    }
}

fn convert_character_class<M: AstInfo>(inp: ast::CharacterClass<M>) -> CharacterClass {
    todo!()
    // match inp {
    //     CharacterClass::Class(_, _) => {}
    // }
}

fn convert_sort<M: AstInfo>(inp: ast::Sort<M>) -> Sort {
    match inp { ast::Sort::Sort(_, name, constructors) => {
        Sort {
            name: convert_identifier(*name),
            constructors: constructors.into_iter().map(|i| convert_constructor(*i)).collect()
        }
    } }
}

fn convert_constructor<M: AstInfo>(inp: ast::Constructor<M>) -> Constructor {
    todo!()
}

use std::fs::File;
use codegen::{Block, Formatter, Function, Impl, Scope};
use crate::codegen::error::CodegenError;
use crate::codegen::sanitize_identifier;
use crate::parser::bootstrap::ast::{Constructor, SyntaxFileAst};
use std::io::Write;

pub fn build_function(name: impl AsRef<str>, contents: impl FnOnce(&mut Function)) -> Function {
    let mut f = Function::new(name.as_ref());

    contents(&mut f);

    f
}

pub fn build_trait_impl(scope: &mut Scope, name: impl AsRef<str>, for_struct: impl AsRef<str>, contents: Vec<Function>) -> &mut Impl {
    let t = scope.new_impl(for_struct.as_ref())
        .impl_trait(name.as_ref());

    for i in contents {
        t.push_fn(i);
    }

    t
}

pub fn match_all_constructors<'a>(
    block: &mut Function,
    constructors: impl IntoIterator<Item = &'a Constructor>,
    match_pattern: impl AsRef<str>,
    mut for_each: impl FnMut(&mut Block, &Constructor),
) {
    let match_pattern = match_pattern.as_ref();

    block.line("match self {");
    for constructor in constructors.into_iter() {
        let mut b = Block::new("");
        for_each(&mut b, constructor);
        let mut res = String::new();
        b.fmt(&mut Formatter::new(&mut res));

        block.line(format!(
            r#"Self::{}{match_pattern} => {res}"#,
            sanitize_identifier(&constructor.name),
        ));
    }
    block.line("}");

}


pub fn write_trait_impls(file: &mut File, syntax: &SyntaxFileAst) -> Result<(), CodegenError> {
    let mut scope = Scope::new();
    scope.import(&format!("super::prelude"), "*");

    for sort in &syntax.sorts {
        let sortname = sanitize_identifier(&sort.name);
        build_trait_impl(&mut scope, "AstNode<M>", format!("{sortname}<M>"), vec![
            build_function("ast_info", |f| {
                match_all_constructors(f, &sort.constructors, "(info, ..)", |b, c| {
                    b.line("info");
                });
                f.arg_ref_self().ret("&M");
            }),

            build_function("constructor", |f| {
                match_all_constructors(f, &sort.constructors, "(info, ..)", |b, c| {
                    b.line(&format!(r#""{}""#, c.name));
                });
                f.arg_ref_self().ret("&'static str");
            }),

            build_function("node_sort", |f| {
                f.line(&format!(r#""{}""#, sort.name));
                f.arg_ref_self().ret("&'static str");
            })
        ]).generic("M: AstInfo");
    }


    write!(file, "{}", scope.to_string())?;

    Ok(())
}
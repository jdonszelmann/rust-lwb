#[macro_export]
macro_rules! language {
    ($name: ident at mod $path: path) => {
        struct $name;

        use $path as AST;

        impl $name {
            fn parse(
                source: &$crate::sources::source_file::SourceFile,
            ) -> AST::AST_ROOT<$crate::parser::ast::generate_ast::BasicAstInfo> {
            }
        }
    };

    ($name: ident at path $path: literal) => {
        paste! {
            #[path = $path]
            mod [<$name _MODULE>];

            language!($name at mod [<$name _MODULE>]);
        }
    };
}

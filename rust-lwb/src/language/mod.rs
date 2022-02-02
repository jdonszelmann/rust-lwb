#[macro_export]
macro_rules! language {
    ($vis: vis $name: ident at mod $path: path) => {
        $vis struct $name;

        use $path as AST;

        impl $name {
            pub fn parse(
                source: &$crate::sources::source_file::SourceFile,
            ) -> Result<AST::AST_ROOT<$crate::parser::ast::generate_ast::BasicAstInfo>, $crate::parser::syntax_file::ParseError> {
                $crate::parser::syntax_file::parse_language(source)
            }
        }
    };

    ($name: ident at mod $path: path) => {
        language!(pub(self) $name at mod $path);
    };
    ($vis: vis $name: ident at path $path: literal) => {
        paste! {
            #[path = $path]
            mod [<$name _MODULE>];

            language!($vis $name at mod [<$name _MODULE>]);
        }
    };
    ($name: ident at path $path: literal) => {
        language!(pub(self) $name at path $path);
    };
}

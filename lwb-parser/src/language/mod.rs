use crate::parser::syntax_file::ParseError;
use crate::sources::source_file::SourceFile;

pub trait Language {
    type Ast;

    /// parses a source file into an AST. Panics (and nicely displays an error)
    /// when the parse failed.
    fn parse(source: &SourceFile) -> Self::Ast {
        match Self::try_parse(source) {
            Ok(i) => i,
            Err(e) => {
                panic!("failed to parse: {e}");
            }
        }
    }

    /// Tries to parse a source file. Returns an error if parsing failed.
    fn try_parse(source: &SourceFile) -> Result<Self::Ast, ParseError>;
}

#[macro_export]
macro_rules! language {
    ($vis: vis $name: ident at mod $path: path) => {
        $vis struct $name;

        use $path as AST;

        impl $crate::language::Language for $name {
            type Ast = AST::AST_ROOT<$crate::parser::ast::generate_ast::BasicAstInfo>;

            fn try_parse(
                source: &$crate::sources::source_file::SourceFile,
            ) -> Result<Self::Ast, $crate::parser::syntax_file::ParseError> {
                $crate::parser::syntax_file::parse_language(source, AST::PARSER)
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

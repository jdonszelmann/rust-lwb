use crate::codegen_prelude::ParsePairSort;
use crate::parser::peg::parse_error::PEGParseError;
use crate::parser::peg::parser_sugar_ast::SyntaxFileAst;
use crate::sources::source_file::SourceFile;

pub fn parse_file<'src>(
    syntax: &'src SyntaxFileAst,
    file: &'src SourceFile,
) -> (ParsePairSort<'src>, Vec<PEGParseError>) {
    todo!()
}
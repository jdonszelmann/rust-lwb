use crate::source_file::SourceFile;

#[derive(Clone)]
#[allow(unused)]
pub struct Span {
    position: usize,
    length: usize,
    source: SourceFile,
}

impl Span {}

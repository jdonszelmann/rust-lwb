use crate::source_file::SourceFile;

#[derive(Clone)]
struct Span {
    position: usize,
    length: usize,
    source: SourceFile,
}

impl Span {}

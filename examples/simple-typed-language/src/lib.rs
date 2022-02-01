use rust_lwb::language;
use rust_lwb::sources::source_file::SourceFile;

mod stl;
mod types;

language!(STL at mod stl);

#[test]
fn test_parse() {
    let file = SourceFile::new("

    ", "main.stl");
    let ast = STL::parse(&file);
}
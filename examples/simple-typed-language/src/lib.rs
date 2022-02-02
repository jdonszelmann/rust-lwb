use rust_lwb::language;

mod stl;
mod types;

language!(STL at mod stl);

#[test]
fn test_parse() {
    use rust_lwb::sources::source_file::SourceFile;
    let file = SourceFile::new(
        "

    ", "main.stl",
    );
    let ast = STL::parse(&file);
}

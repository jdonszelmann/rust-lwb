use rust_lwb::language;
use rust_lwb::language::Language;
use rust_lwb::sources::source_file::SourceFile;
use rust_lwb::typechecker::TypeChecker;

#[rustfmt::ignore]
mod stl;
mod types;

language!(STL at mod stl);

#[test]
fn test_parse() {
    let file = SourceFile::new("
3 + 5;
    ", "main.stl");
    let ast = match STL::parse(&file) {
        Ok(ok) => ok,
        Err(e) => {
            println!("{}", e);
            panic!();
        },
    };

    let tc = TypeChecker::new();
    tc.check_types(ast, &());
}
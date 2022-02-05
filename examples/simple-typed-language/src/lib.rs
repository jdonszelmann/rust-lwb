use rust_lwb::language;

#[rustfmt::ignore]
mod stl;
mod types;

language!(STL at mod stl);

#[cfg(test)]
mod tests {
    use crate::STL;
    use rust_lwb::language::Language;
    use rust_lwb::sources::source_file::SourceFile;
    use rust_lwb::typechecker::TypeChecker;

    #[test]
    fn test_parse() {
        let file = SourceFile::new(
            "
3 + 3;
    ",
            "main.stl",
        );
        let ast = match STL::parse(&file) {
            Ok(ok) => ok,
            Err(e) => {
                println!("{}", e);
                panic!();
            }
        };

        let tc = TypeChecker::new();
        let tres = tc.check_types(ast, &());
        assert!(tres.is_ok(), "{}", tres.unwrap_err());
    }
}

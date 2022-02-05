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

    macro_rules! test_stl {
        ($name: ident: $input: literal $($tt: tt)*) => {
            #[test]
            fn $name() {
                let file = SourceFile::new($input, "main.stl");

                macro_rules! parse_test_type {
                    (should not parse) => {
                        let res = STL::parse(&file);
                        assert!(res.is_err(), "{:?}", res.unwrap());
                    };
                    (should typecheck) => {
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
                    };
                    (should not typecheck) => {
                        let ast = match STL::parse(&file) {
                            Ok(ok) => ok,
                            Err(e) => {
                                println!("{}", e);
                                panic!();
                            }
                        };
                        let tc = TypeChecker::new();
                        let tres = tc.check_types(ast, &());
                        assert!(tres.is_err());
                    }
                }
                parse_test_type!($($tt)*);
            }

        };
    }

    test_stl!(addition_ok: "3 + 3;" should typecheck);
    test_stl!(addition_err_1: "3 + true;" should not typecheck);
    test_stl!(addition_err_2: "true + 3;" should not typecheck);
}

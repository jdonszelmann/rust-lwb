#![cfg_attr(feature = "nightly", feature(box_patterns))]

#[cfg(not(feature = "nightly"))]
#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        eprintln!("=====================================================");
        eprintln!("|| USE A NIGHTLY VERSION OF RUST TO RUN MORE TESTS ||");
        eprintln!("=====================================================");
    }
}

#[cfg(feature = "nightly")]
#[cfg(test)]
mod tests {
    macro_rules! should_parse_tests {
        ($case: literal to $expr: pat, $($rest: tt)*) => {
            let file = SourceFile::new($case, "main.lang");
            let res = LangImpl::parse(&file);
            assert!(res.is_ok(), "{}", res.unwrap_err());
            let val_str = format!("{:?}", res.as_ref().unwrap());
            assert!(matches!(res.unwrap(), $expr), "{:?} did not match {}", val_str, stringify!($expr));

            should_parse_tests!($($rest)*);
        };
        ($case: literal, $($rest: tt)*) => {
            let file = SourceFile::new($case, "main.lang");
            let res = LangImpl::parse(&file);
            assert!(res.is_ok(), "{}", res.unwrap_err());

            should_parse_tests!($($rest)*);
        };
        () => {}
    }

    macro_rules! should_not_parse_tests {
        ($case: literal, $($rest: tt)*) => {
            let file = SourceFile::new($case, "main.lang");
            let res = LangImpl::parse(&file);
            assert!(res.is_err(), "expected error, but got Ok: {:?}", res.unwrap());

            should_not_parse_tests!($($rest)*);
        };
        () => {}
    }

    macro_rules! test {
        (
            name: $name: ident $(,)?
            non_exhaustive: $non_exhaustive: literal $(,)?
            grammar: $grammar: literal $(,)?
            $(should parse: [$($should_parse: tt)*])? $(,)?
            $(should not parse: [$($should_not_parse: tt)*])? $(,)?
            $(attrs: $($attrs:tt)*)? $(,)?
        ) => {
            mod $name {
                use rust_lwb::language;
                use rust_lwb::sources::source_file::SourceFile;
                use rust_lwb::language::Language;

                mod lang {
                    use rust_lwb_macros::generate;

                    generate!($grammar, $non_exhaustive);
                }

                pub use lang::*;

                language!(LangImpl at mod lang);

                #[test]
                $($attrs)*
                fn $name() {
                    $(
                        should_parse_tests!($($should_parse)*);
                    )?
                    $(
                        should_not_parse_tests!($($should_not_parse)*);
                    )?
                }
            }
        };
    }

    test!(
        name: simple_grammar,
        non_exhaustive: false,
        grammar: r#"
As:
    More = "a" As;
    NoMore = "";
start at As;
        "#,
        should parse: [
            "aaa",
            "" to As::NoMore(..),
            "a" to As::More(_, box As::NoMore(..)),
        ]
        should not parse: [
            "b",
            "bb",
        ]
    );

    test!(
        name: non_exhaustive,
        non_exhaustive: true,
        grammar: r#"
As:
    More = "a" As;
    NoMore = "";
start at As;
        "#,
        should parse: [
            "aaa",
            "" to As::NoMore(..),
            "a" to As::More(_, box As::NoMore(..), ..),
        ]
        should not parse: [
            "b",
            "bb",
        ]
    );
}

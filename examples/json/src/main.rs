use lwb_parser::language;

#[rustfmt::skip]
mod ast;

language!(JSON at mod ast);

fn main() {}

#[cfg(test)]
mod tests {
    use super::JSON;
    use lwb_parser::language::Language;
    use lwb_parser::sources::source_file::SourceFile;

    macro_rules! json_test {
        (err: $src: literal) => {
            let r = JSON::parse(&SourceFile::new($src, "test.json"));
            assert!(r.is_err(), "{:?}", r.unwrap());
        };
        ($src: literal) => {
            let r = JSON::parse(&SourceFile::new($src, "test.json"));
            assert!(r.is_ok(), "{}", r.unwrap_err());
        };
    }

    #[test]
    fn parse_json() {
        json_test!("null");
        json_test!("true");
        json_test!("false");
        json_test!("1");
        json_test!("[true, false, null]");
        json_test!("[]");
        json_test!(r#"{"test": true}"#);
        json_test!("{}");
    }
}

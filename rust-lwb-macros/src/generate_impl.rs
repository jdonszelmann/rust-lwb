use proc_macro::TokenStream;
use quote::quote;
use rust_lwb::codegen::manager::__codegen_tokenstream;
use rust_lwb::config::{Config, LanguageConfig, SyntaxConfig};
use rust_lwb::sources::source_file::SourceFile;
use syn::{parse_macro_input, LitStr};

pub fn generate(input: TokenStream) -> TokenStream {
    let i: LitStr = parse_macro_input!(input as LitStr);

    let sf = SourceFile::new(i.value(), "test.syntax");
    let cfg = Config {
        syntax: SyntaxConfig {
            destination: "".to_string(),
            definition: "".to_string(),
            non_exhaustive: false,
            serde: false,
            import_location: "rust_lwb".to_string(),
            write_serialized_ast: true,
        },
        language: LanguageConfig {
            name: "test".to_string(),
            extensions: vec![],
        },
    };

    match __codegen_tokenstream(sf, cfg, true) {
        Ok(i) => i.into(),
        Err(e) => {
            let e = e.to_string();
            quote!(
                compile_error!("{}", #e);
            )
            .into()
        }
    }
}

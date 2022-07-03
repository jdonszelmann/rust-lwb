use proc_macro::TokenStream;
use quote::quote;
use rust_lwb::codegen::manager::__codegen_tokenstream;
use rust_lwb::config::{Config, LanguageConfig, SyntaxConfig};
use rust_lwb::sources::source_file::SourceFile;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, LitBool, LitStr, Token};

struct MacroInput {
    grammar: LitStr,
    non_exhaustive: LitBool,
    serde: LitBool,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let grammar = input.parse()?;
        input.parse::<Token![,]>()?;
        let non_exhaustive = input.parse()?;
        input.parse::<Token![,]>()?;
        let serde = input.parse()?;

        Ok(Self {
            grammar,
            non_exhaustive,
            serde,
        })
    }
}

pub fn generate(input: TokenStream) -> TokenStream {
    let i = parse_macro_input!(input as MacroInput);

    let sf = SourceFile::new(i.grammar.value(), "test.syntax");
    let cfg = Config {
        syntax: SyntaxConfig {
            destination: "".to_string(),
            definition: "".to_string(),
            non_exhaustive: i.non_exhaustive.value,
            serde: i.serde.value,
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

use proc_macro::TokenStream;
use quote::quote;
use rust_lwb::codegen::manager::CodegenManager;
use syn::{parse_macro_input, LitStr};

pub fn generate(input: TokenStream) -> TokenStream {
    let i: LitStr = parse_macro_input!(input as LitStr);

    let mut cm = CodegenManager::new();
    cm.__add_syntax_str(i.value(), "test.syntax");

    match cm.__codegen_tokenstream(true) {
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

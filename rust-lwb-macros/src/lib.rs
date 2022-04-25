use proc_macro::TokenStream;

mod generate_impl;

#[proc_macro]
pub fn generate(input: TokenStream) -> TokenStream {
    generate_impl::generate(input)
}


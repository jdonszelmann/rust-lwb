use crate::codegen::error::CodegenError;
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn generate_root(
    names: &[&str],
    derives: &[&str],
    prelude_import_location: &str,
) -> Result<TokenStream, CodegenError> {
    let names = names.iter().map(|i| format_ident!("{}", i)).collect_vec();
    let prelude_import_location = format_ident!("{}", prelude_import_location);
    let derives = derives.iter().map(|i| format_ident!("{}", i)).collect_vec();

    Ok(quote!(
        #(
            #[rustfmt::skip]
            mod #names;
            pub use #names::*;
        )*

        #[rustfmt::skip]
        mod prelude {
            pub use #prelude_import_location::codegen_prelude::*;
            pub use super::*;

            /// This type is public, but in a private module. That means nothing can ever construct
            /// this value except from within this module. This ensures that AST types can only be
            /// constructed by the parser, and that matches on types containing this must be
            /// non-exhaustive. Rust does have the #[non_exhaustive] attribute, but it only works
            /// between crate boundaries, not within the same crate which is what this enforces.
            #[derive(Copy, Clone, PartialEq, #(#derives),*)]
            pub struct NonExhaustive;
        }
    ))
}

pub fn generate_parser(parser: &[u8]) -> Result<TokenStream, CodegenError> {
    Ok(quote!(
        pub const PARSER: &[u8] = &[#(#parser),*];
    ))
}

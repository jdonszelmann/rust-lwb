use crate::codegen::error::CodegenError;
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// When re-exporting serde, it doesn't automatically work with derive.
/// See this comment: https://github.com/serde-rs/serde/issues/1465#issuecomment-800686252
///
/// Therefore, when serde is detected in the derives, we need to emit some more attributes
pub fn generate_serde_attrs(derives: &[&str]) -> TokenStream {
    if derives.contains(&"Serialize") || derives.contains(&"Deserialize") {
        quote!(
           #[serde(crate = "self::serde")]
        )
    } else {
        TokenStream::new()
    }
}

pub fn generate_root(
    names: &[&str],
    derives: &[&str],
    prelude_import_location: &str,
    non_exhaustive: bool,
) -> Result<TokenStream, CodegenError> {
    let serde_attrs = generate_serde_attrs(derives);

    let names = names.iter().map(|i| format_ident!("{}", i)).collect_vec();
    let prelude_import_location = format_ident!("{}", prelude_import_location);
    let derives = derives.iter().map(|i| format_ident!("{}", i)).collect_vec();

    let non_exhaustive = if non_exhaustive {
        quote!(
            /// This type is public, but in a private module. That means nothing can ever construct
            /// this value except from within this module. This ensures that AST types can only be
            /// constructed by the parser, and that matches on types containing this must be
            /// non-exhaustive. Rust does have the #[non_exhaustive] attribute, but it only works
            /// between crate boundaries, not within the same crate which is what this enforces.
            #[derive(Copy, Clone, #(#derives),*)]
            #serde_attrs
            pub struct NonExhaustive;
        )
    } else {
        TokenStream::new()
    };

    Ok(quote!(
        #(
            mod #names;
            pub use #names::*;
        )*

        mod prelude {
            pub use #prelude_import_location::codegen_prelude::*;
            pub use super::*;

            #non_exhaustive
        }
    ))
}

pub fn generate_parser(parser: &[u8]) -> Result<TokenStream, CodegenError> {
    Ok(quote!(
        pub const PARSER: &[u8] = &[#(#parser),*];
    ))
}

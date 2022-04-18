use convert_case::{Case, Casing};

mod error;
pub mod generate_ast;
mod generate_from_pairs;
mod generate_headers;
mod generate_trait_impls;
pub mod manager;

fn sanitize_identifier(id: &str) -> String {
    id.to_case(Case::UpperCamel)
}

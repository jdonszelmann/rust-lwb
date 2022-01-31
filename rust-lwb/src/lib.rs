extern crate paste;

/// Code describing languages created with rust-lwb
#[macro_use]
pub mod language;

/// Helpers for type checking
pub mod typechecker;

/// Helpers for code generation
pub mod transform;

/// Contains code related to syntax definitions
/// and parsing target languages based on these
/// definitions. Also contains sort/constructor
/// related code and code generation for these.
/// Contains the PEG parser.
pub mod parser;

/// Code related to generating rust source files
/// from language definitions. Usually used from
/// build.rs files.
pub mod codegen;

/// Contains code related to source code of languages
/// such as spans, and the [`SourceFile`] struct.
pub mod sources;

/// Collection of imports used in automatically generated
/// files (to avoid listing many imports in them). Should
/// not generally be used directly by users of rust-lwb
pub mod codegen_prelude;

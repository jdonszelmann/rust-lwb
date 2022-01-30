/// Contains code related to syntax definitions
/// and parsing target languages based on these
/// definitions. Also contains sort/constructor
/// related code and code generation for these.
/// Contains the PEG parser.
pub mod parser;

/// Helpers for type checking
pub mod types;

/// Helpers for code generation
pub mod transform;

/// Code to deal with source files (both used
/// internally and for source of custom defined
/// languages). Contains [`Span`] and [`SourceFile`]
pub mod source;

/// Code related to generating rust source files
/// from language definitions. Usually used from
/// build.rs files.
pub mod codegen;

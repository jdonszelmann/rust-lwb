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

/// Source files (stored together with a name).
pub mod source_file;
/// Code spans (which reference to a source file)
pub mod span;

/// Code related to generating rust source files
/// from language definitions. Usually used from
/// build.rs files.
pub mod codegen;

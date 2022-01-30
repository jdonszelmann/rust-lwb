/// Contains parse errors and methods for
/// dealing with them.
pub mod error;

/// Contains code related to parsing syntax
/// definition files
pub mod syntax_file;

/// Contains code related to abstract syntax
/// trees generated from the syntax definitions
/// for user-defined languages.
pub mod ast;

/// Contains code related to the peg parser.
pub mod peg;

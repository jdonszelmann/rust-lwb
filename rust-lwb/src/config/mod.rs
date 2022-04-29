use serde::Deserialize;

pub mod toml;

fn default_import_location() -> String {
    "rust_lwb".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
pub struct SyntaxConfig {
    /// The location (path) where the generated code should be
    /// written, relative to the configuration file
    pub destination: String,

    /// The path to the syntax definition file, relative to the
    /// configuration file
    pub definition: String,

    /// Make AST items have the non-exhaustive attribute. This
    /// improves backwards compatibility, but reduces the number
    /// of checks the rust compiler can do on your code using the
    /// AST (ie. you can make more variants, but not handling the
    /// new variant is not an error with this on).
    ///
    /// Set to whichever you prefer.
    #[serde(default)]
    pub non_exhaustive: bool,

    /// Derive serde traits for the AST types
    #[serde(default)]
    pub serde: bool,

    /// The location to import rust_lwb from. Defaults to "rust_lwb"
    #[serde(default = "default_import_location")]
    pub import_location: String,

    #[doc(hidden)]
    #[serde(default = "default_true")]
    pub write_serialized_ast: bool, // always true except during bootstrap.
}

#[derive(Deserialize)]
pub struct LanguageConfig {
    /// The name of your language
    pub name: String,

    /// The different extensions associated with your language
    #[serde(default)]
    pub extensions: Vec<String>,
}

#[derive(Deserialize)]
pub struct Config {
    pub syntax: SyntaxConfig,
    pub language: LanguageConfig,
}

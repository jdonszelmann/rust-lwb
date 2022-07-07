use serde::Deserialize;

pub mod toml;

fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
pub enum Mode {
    #[serde(rename = "lwb")]
    Lwb,
    #[serde(rename = "parser")]
    Parser,
    #[serde(rename = "parser")]
    Custom(String),
}

impl Mode {
    pub fn import_location(&self) -> &str {
        match self {
            Mode::Lwb => "rust_lwb",
            Mode::Parser => "lwb_parser",
            Mode::Custom(a) => a,
        }
    }
}

impl Default for Mode {
    fn default() -> Self {
        Self::Parser
    }
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

    /// The mode to work in. Defaults to parser since rust_lwb is far from done.
    /// This also sets import_location to lwb_parser
    #[serde(default)]
    pub mode: Mode,

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

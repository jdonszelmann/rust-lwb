use crate::config::Config;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Find the location of the LWB config.
/// This function should only be run from build scripts.
pub fn find_config_path() -> PathBuf {
    let p = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("couldn't find cargo manifest dir"),
    );
    p.join("lwb.toml")
}

#[derive(Debug, Error)]
pub enum ParseConfigError {
    #[error("toml deserialize error: {0}")]
    Toml(#[from] toml::de::Error),
}

#[derive(Debug, Error)]
pub enum ReadConfigError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("parse error: {0}")]
    Parse(#[from] ParseConfigError),
}

pub fn read_config(path: impl AsRef<Path>) -> Result<Config, ReadConfigError> {
    let f = std::fs::read_to_string(path)?;

    Ok(parse_config(f)?)
}

pub fn parse_config(config_str: impl AsRef<str>) -> Result<Config, ParseConfigError> {
    let res = toml::from_str(config_str.as_ref())?;

    Ok(res)
}

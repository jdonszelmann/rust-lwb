use std::error::Error;
use std::fmt::Display;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct BootstrapConfig {
    pub input_location: String,
    pub output_location: String,
    pub serialized_ast_location: String,
}

pub fn from_root(path_from_root: impl AsRef<Path>) -> PathBuf {
    let mut f = root().clone();
    f.push(path_from_root);
    f
}

pub fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR").to_string())
}

pub fn load(path_from_root: impl AsRef<Path>) -> BootstrapConfig {
    let f = from_root(path_from_root);

    let mut file = File::open(f).expect("failed to open config file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("failed to read config file");

    toml::from_str(&contents).expect("failed to parse config file")
}

pub fn unwrap<T, E: Error + Display>(e: Result<T, E>) -> T{
    if let Err(e) = e {
        panic!("{}", e);
    } else {
        e.unwrap()
    }
}
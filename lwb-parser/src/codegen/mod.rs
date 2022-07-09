use convert_case::{Case, Casing};
use regex::Captures;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, io};

mod check_recursive;
mod error;
mod generate_file_headers;
mod generate_from_pairs;
mod generate_misc;
mod generate_structs;
mod generate_trait_impls;
pub mod manager;

fn sanitize_identifier(id: &str) -> String {
    id.to_case(Case::UpperCamel)
}

/// Like a file, formats contents on closing
pub struct FormattingFile(Option<File>, PathBuf);

impl FormattingFile {
    pub fn create(p: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Self(Some(File::create(&p)?), p.as_ref().to_path_buf()))
    }

    pub fn open(p: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Self(Some(File::open(&p)?), p.as_ref().to_path_buf()))
    }
}

fn try_fmt(p: impl AsRef<Path>) -> io::Result<()> {
    println!("Formatting {:?}", p.as_ref());
    Command::new("rustfmt").arg(p.as_ref()).spawn()?.wait()?;

    let r = regex::Regex::new(r#"#\[doc *= *"(.*)"\]"#).expect("should compile");
    let rq = regex::Regex::new(r#"\\(.)"#).expect("should compile");

    let code = fs::read_to_string(&p)?;
    let replaced = r.replace_all(&code, |caps: &Captures| {
        format!("///{}", rq.replace_all(&caps[1], "$1"))
    });
    fs::write(&p, replaced.as_ref())?;

    Ok(())
}

impl Drop for FormattingFile {
    fn drop(&mut self) {
        drop(self.0.take());

        if let Err(e) = try_fmt(&self.1) {
            eprintln!("{}", e);
        }
    }
}

impl Read for FormattingFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.as_ref().unwrap().read(buf)
    }
}

impl Write for FormattingFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.as_ref().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.as_ref().unwrap().flush()
    }
}

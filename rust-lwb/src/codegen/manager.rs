use crate::codegen::generate_language;
use crate::parser::bootstrap::parse;
use crate::parser::bootstrap::ParseError;
use crate::sources::source_file::SourceFile;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;
use crate::parser::syntax_file::convert_syntax_file_ast;

#[derive(Debug, Error)]
pub enum CodegenError {
    #[error("An io error occurred: {0}")]
    Io(#[from] std::io::Error),

    #[error("a parse error occurred: {0}")]
    ParseError(#[from] ParseError),
}

pub struct CodeGenJob {
    location: PathBuf,
    destination: PathBuf,
    import_location: String,

    /// Make Ast serializable
    serde: bool,
}

impl CodeGenJob {
    pub fn from_path(p: PathBuf) -> CodeGenJob {
        let mut destination = p.clone();
        destination.set_extension("rs");
        let name = destination.file_name().expect("no file name in path");
        let new_filename = name.to_string_lossy().replace('-', "_");
        destination.set_file_name(new_filename);

        Self {
            location: p,
            destination,
            import_location: "rust_lwb".to_string(),
            serde: false,
        }
    }

    pub fn destination(&mut self, p: impl AsRef<Path>) -> &mut Self {
        self.destination = p.as_ref().to_path_buf();
        self
    }

    pub fn import_location(&mut self, path: impl AsRef<str>) -> &mut Self {
        self.import_location = path.as_ref().to_string();
        self
    }

    pub fn serde(&mut self, enable_serde: bool) -> &mut Self {
        self.serde = enable_serde;
        self
    }


    pub fn codegen(self) -> Result<(), CodegenError> {
        let sf = SourceFile::open(self.location)?;
        let ast = parse(&sf)?; // TODO: replace with bootstrapped parser

        let res = generate_language(ast, &self.import_location, self.serde);

        let mut res_file = std::fs::File::create(self.destination)?;
        res_file.write_all(res.as_bytes())?;

        Ok(())
    }
}

pub struct CodegenManager {
    jobs: Vec<CodeGenJob>,
}

impl CodegenManager {
    pub fn new() -> Self {
        Self { jobs: vec![] }
    }

    pub fn add_syntax_file(&mut self, path: impl AsRef<Path>) -> &mut CodeGenJob {
        self.jobs
            .push(CodeGenJob::from_path(path.as_ref().to_path_buf()));

        self.jobs
            .last_mut()
            .expect("must always be a last item since we just pushed something")
    }

    pub fn codegen(self) -> Result<(), CodegenError> {
        for job in self.jobs {
            println!("cargo:rerun-if-changed={:?}", job.location);
            job.codegen()?;
        }

        Ok(())
    }
}

impl Default for CodegenManager {
    fn default() -> Self {
        Self::new()
    }
}

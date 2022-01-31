use std::io::Write;
use std::path::{Path, PathBuf};
use crate::codegen::generate_language;
use crate::parser::bootstrap::parse;
use crate::sources::source_file::SourceFile;
use thiserror::Error;
use crate::parser::bootstrap::ParseError;

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
}

impl CodeGenJob {
    pub fn from_path(p: PathBuf) -> CodeGenJob {
        let mut destination = p.clone();
        destination.set_extension(".rs");
        Self {
            location: p,
            destination
        }
    }

    pub fn destination(&mut self, p: impl AsRef<Path>) -> &mut Self {
        self.destination = p.as_ref().to_path_buf();
        self
    }

    pub fn codegen(self) -> Result<(), CodegenError> {
        let sf = SourceFile::open(self.location)?;
        let ast = parse(&sf)?;

        let res = generate_language(ast);

        let mut res_file = std::fs::File::create(self.destination)?;
        res_file.write_all(res.as_bytes())?;

        Ok(())
    }
}

pub struct CodegenManager {
    jobs: Vec<CodeGenJob>
}

impl CodegenManager {
    pub fn new() -> Self {
       Self {
           jobs: vec![]
       }
    }

    pub fn add_syntax_file(&mut self, path: impl AsRef<Path>) -> &mut CodeGenJob {
        self.jobs.push(CodeGenJob::from_path(path.as_ref().to_path_buf()));

        self.jobs.last_mut().expect("must always be a last item since we just pushed something")
    }


    pub fn codegen(self) -> Result<(), CodegenError> {
        for job in self.jobs {
            job.codegen()?;
        }

        Ok(())
    }
}


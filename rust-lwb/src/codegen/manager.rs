use crate::codegen::generate_language;
use crate::sources::source_file::SourceFile;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;
use crate::language::Language;
// use crate::parser::bootstrap::parse;
use crate::parser::syntax_file::convert_syntax_file_ast::AstConversionError;
use crate::parser::syntax_file::{convert_syntax_file_ast, ParseError, SyntaxFile};

#[derive(Debug, Error)]
pub enum CodegenError {
    #[error("An io error occurred: {0}")]
    Io(#[from] std::io::Error),

    #[error("a parse error occurred: {0}")]
    ParseError(#[from] ParseError),

    #[error("failed to convert saved syntax file definition ast to legacy syntax file definition ast (this is a bug! please report it)")]
    ConvertAstError(#[from] AstConversionError),

    #[error("failed to serialize parser")]
    Bincode(#[from] bincode::Error)
}

pub struct CodeGenJob {
    location: PathBuf,
    destination: PathBuf,
    import_location: String,

    /// Make Ast serializable
    serde: bool,

    #[doc(hidden)]
    pub write_serialized_ast: bool // always true except bootstrap.
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
            write_serialized_ast: true,
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

    pub fn dont_generate_serialized_ast(&mut self) -> &mut Self {
        self.write_serialized_ast = false;
        self
    }

    pub fn serde(&mut self, enable_serde: bool) -> &mut Self {
        self.serde = enable_serde;
        self
    }

    pub fn codegen(self) -> Result<(), CodegenError> {
        let sf = SourceFile::open(self.location)?;
        let ast = SyntaxFile::parse(&sf)?;

        let serialized_parser = bincode::serialize(&ast)?;

        let legacy_ast = convert_syntax_file_ast::convert(ast)?; // TODO make peg parser use new ast

        // TODO: initial bootstrap (remove)
        // let sf = SourceFile::open(&self.location)?;
        // let legacy_ast = parse(&sf).expect("should parse");

        let serialized_parser = if self.write_serialized_ast {
            Some(serialized_parser.as_slice())
        } else {
            None
        };

        let res = generate_language(legacy_ast, &self.import_location, self.serde, serialized_parser);

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

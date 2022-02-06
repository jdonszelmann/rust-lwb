use crate::codegen::generate_headers::write_headers;
use std::fs::File;
// use crate::codegen::generate_language;
use crate::codegen::error::CodegenError;
use crate::codegen::error::CodegenError::NoExtension;
use crate::codegen::generate_ast::write_ast;
use crate::codegen::generate_from_pairs::write_from_pairs;
use crate::codegen::generate_trait_impls::write_trait_impls;
use crate::language::Language;
use crate::parser::syntax_file::{convert_syntax_file_ast, SyntaxFile};
use crate::sources::source_file::SourceFile;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct CodeGenJob {
    location: PathBuf,
    destination: PathBuf,
    import_location: String,

    /// Make Ast serializable
    serde: bool,

    #[doc(hidden)]
    pub write_serialized_ast: bool, // always true except bootstrap.
}

pub fn create_module_files<const N: usize>(
    location: impl AsRef<Path>,
    files: [&str; N],
) -> Result<[(File, String); N], CodegenError> {
    let mut location = location.as_ref().to_path_buf();
    location.set_extension("");

    std::fs::create_dir_all(&location)?;
    println!("cargo:rerun-if-changed={:?}", location);

    let mut res = files.map(|_| None);
    for (index, &i) in files.iter().enumerate() {
        let mut filename = location.clone();
        filename.push(i);
        filename.set_extension("rs");

        println!("cargo:rerun-if-changed={:?}", filename);
        res[index] = Some((
            File::create(&filename)?,
            filename
                .file_stem()
                .ok_or(NoExtension)?
                .to_string_lossy()
                .into_owned(),
        ));
    }

    Ok(res.map(|i| i.unwrap()))
}

fn refs(inp: &mut (File, String)) -> (&mut File, &str) {
    (&mut inp.0, &inp.1)
}

impl CodeGenJob {
    pub(crate) fn from_path(p: PathBuf) -> CodeGenJob {
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

    /// Set the location where files for this job are to be generated.
    ///
    /// Defaults to the same path as the syntax definition file, but whithout
    /// the `.syntax` extension.
    pub fn destination(&mut self, p: impl AsRef<Path>) -> &mut Self {
        self.destination = p.as_ref().to_path_buf();
        self
    }

    /// Set the location where the generated code can import `rust_lwb` from.
    ///
    /// The default will usually be right for you, as it defaults to `rust_lwb`.
    /// As long as `rust_lwb` is a direct dependency of the project the code is generated
    /// in this works.
    ///
    /// For internal use in `rust_lwb` this parameter needs to be set to `crate`
    pub fn import_location(&mut self, path: impl AsRef<str>) -> &mut Self {
        self.import_location = path.as_ref().to_string();
        self
    }

    #[doc(hidden)]
    pub fn dont_generate_serialized_ast(&mut self) -> &mut Self {
        self.write_serialized_ast = false;
        self
    }

    /// Derive Serde's `Serialize` and `Deserialize` for AST enums.
    pub fn serde(&mut self, enable_serde: bool) -> &mut Self {
        self.serde = enable_serde;
        self
    }

    pub(crate) fn codegen(self) -> Result<(), CodegenError> {
        let sf = SourceFile::open(self.location)?;
        let ast = SyntaxFile::parse(&sf)?;

        let serialized_parser = bincode::serialize(&ast)?;

        let legacy_ast = convert_syntax_file_ast::convert(ast)?; // TODO make peg parser use new ast

        let [mut modrs, mut rest @ ..] = create_module_files(
            self.destination,
            [
                "mod.rs",
                "ast.rs",
                "from_pairs.rs",
                "ast_impls.rs",
                "parser.rs",
            ],
        )?;

        write_headers(
            &mut modrs.0,
            &mut rest.iter_mut().map(refs).collect::<Vec<_>>(),
            &self.import_location,
        )?;

        let [ref mut f_ast, ref mut f_from_pairs, ref mut f_ast_trait_impls, ref mut f_serialized_parser] =
            rest.map(|i| i.0);

        let mut derives = vec!["Debug"];
        if self.serde {
            derives.extend(["Serialize, Deserialize"]);
        }

        write_ast(f_ast, &legacy_ast, &derives)?;
        write_from_pairs(f_from_pairs, &legacy_ast)?;
        write_trait_impls(f_ast_trait_impls, &legacy_ast)?;
        if self.write_serialized_ast {
            write!(
                f_serialized_parser,
                r##"pub const PARSER: &[u8] = &{:?};"##,
                serialized_parser
            )?;
        }

        Ok(())
    }
}

/// The CodegenManager manages one or more code generation jobs.
/// Jobs can be submitted through [`add_syntax_file`]. Currently,
/// the only thing that requires code generation is converting
/// syntax definition files into asts. This may change in the future.
pub struct CodegenManager {
    jobs: Vec<CodeGenJob>,
}

impl CodegenManager {
    /// Create a new code generation manager.
    pub fn new() -> Self {
        Self { jobs: vec![] }
    }

    /// Add the location of a syntax definition file to the manager. When [`codegen`]
    /// is called, all syntax definition files are parsed and asts are created.
    ///
    /// Syntax definition files usually use the `.syntax` extension, though any extension
    /// works. Make sure that there is at least *an* extension since by default the location
    /// for generated files is the same as the name of the syntax definition file without the
    /// extension.
    ///
    /// A mutable reference to a [`CodeGenJob`] is returned so you can configure
    /// code generation parameters for this specific syntax definition file.
    ///
    /// See [`CodeGenJob`] for more details on options for code generation.
    pub fn add_syntax_file(&mut self, path: impl AsRef<Path>) -> &mut CodeGenJob {
        self.jobs
            .push(CodeGenJob::from_path(path.as_ref().to_path_buf()));

        self.jobs
            .last_mut()
            .expect("must always be a last item since we just pushed something")
    }

    /// Execute all code generation jobs.
    pub fn codegen(self) -> Result<(), CodegenError> {
        for job in self.jobs {
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

use std::io;
use crate::codegen::generate_file_headers::write_headers;
// use crate::codegen::generate_language;
use crate::codegen::error::CodegenError;
use crate::codegen::error::CodegenError::NoExtension;
use crate::codegen::generate_structs::generate_structs;
use crate::codegen::generate_from_pairs::generate_from_pairs;
use crate::codegen::generate_trait_impls::generate_trait_impls;
use crate::codegen::FormattingFile;
use crate::language::Language;
use crate::parser::syntax_file::{convert_syntax_file_ast, SyntaxFile};
use crate::sources::source_file::SourceFile;
use std::io::Write;
use std::path::{Path, PathBuf};
use proc_macro2::TokenStream;
use quote::quote;
use crate::codegen::generate_misc::{generate_parser, generate_root};

pub struct CodeGenJob {
    source: Result<SourceFile, io::Error>,
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
) -> Result<[(FormattingFile, String); N], CodegenError> {
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
            FormattingFile::create(&filename)?,
            filename
                .file_stem()
                .ok_or(NoExtension)?
                .to_string_lossy()
                .into_owned(),
        ));
    }

    Ok(res.map(|i| i.unwrap()))
}

fn refs(inp: &mut (FormattingFile, String)) -> (&mut FormattingFile, &str) {
    (&mut inp.0, &inp.1)
}

pub(crate) struct Generated {
    impls: TokenStream,
    structs: TokenStream,
    from_pairs: TokenStream,
    root: TokenStream,
    parser: TokenStream,
}

impl CodeGenJob {
    pub(crate) fn from_str(contents: impl AsRef<str>, p: impl AsRef<Path>) -> CodeGenJob {
        let mut destination = p.as_ref().to_path_buf();
        destination.set_extension("rs");
        let name = destination.file_name().expect("no file name in path");
        let new_filename = name.to_string_lossy().replace('-', "_");
        destination.set_file_name(new_filename);

        Self {
            source: Ok(SourceFile::new(contents, p.as_ref().to_string_lossy())),
            destination,
            import_location: "rust_lwb".to_string(),
            serde: false,
            write_serialized_ast: true,
        }
    }

    pub(crate) fn from_path(p: PathBuf) -> CodeGenJob {
        let mut destination = p.clone();
        destination.set_extension("rs");
        let name = destination.file_name().expect("no file name in path");
        let new_filename = name.to_string_lossy().replace('-', "_");
        destination.set_file_name(new_filename);

        Self {
            source: SourceFile::open(p),
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

    #[doc(hidden)]
    pub fn __codegen_tokenstream(self, debug: bool) -> Result<TokenStream, CodegenError> {
        assert!(self.write_serialized_ast, "can only codegen tokenstream with serialized ast on");

        let Generated {
            impls, structs, from_pairs, root, parser
        } = self.codegen_internal(&[])?;

        if debug {
            println!("{}", structs);
        }

        Ok(quote!(
            mod ast {
                #structs
            }
            mod from_pairs {
                #from_pairs
            }
            mod ast_impls {
                #impls
            }
            mod parser {
                #parser
            }

            pub use ast::*;
            pub use from_pairs::*;
            pub use ast_impls::*;
            pub use parser::*;

            #root
        ))
    }

    fn codegen_internal(self, imports: &[&str]) -> Result<Generated, CodegenError> {
        let ast = SyntaxFile::parse(&self.source?)?;

        let serialized_parser = bincode::serialize(&ast)?;

        let legacy_ast = convert_syntax_file_ast::convert(ast)?; // TODO make peg parser use new ast

        let mut derives = vec!["Debug"];
        if self.serde {
            derives.extend(["Serialize", "Deserialize"]);
        }

        let structs = generate_structs(&legacy_ast, &derives)?;
        let from_pairs = generate_from_pairs(&legacy_ast)?;
        let impls = generate_trait_impls(&legacy_ast)?;
        let root = generate_root(imports, &derives, &self.import_location)?;
        let parser = generate_parser(&serialized_parser)?;

        Ok(Generated {
            impls,
            structs,
            from_pairs,
            root,
            parser,
        })
    }

    pub(crate) fn codegen(self) -> Result<(), CodegenError> {
        let mut files = create_module_files(
            &self.destination,
            [
                "mod.rs",
                "ast.rs",
                "from_pairs.rs",
                "ast_impls.rs",
                "parser.rs",
            ],
        )?;

        write_headers(&mut files.iter_mut().map(refs).collect::<Vec<_>>())?;

        let [ref mut f_modrs, ref mut f_ast, ref mut f_from_pairs, ref mut f_ast_trait_impls, ref mut f_serialized_parser] =
            files.map(|i| i.0);

        let write_serialized_ast = self.write_serialized_ast;

        let Generated {
            impls, structs, from_pairs, root, parser
        } = self.codegen_internal(&["ast", "from_pairs", "ast_impls", "parser"])?;

        write!(f_modrs, "{}", root)?;
        write!(f_ast, "{}", structs)?;
        write!(f_ast_trait_impls, "{}", impls)?;
        write!(f_from_pairs, "{}", from_pairs)?;

        if write_serialized_ast {
            write!(f_serialized_parser, "{}", parser)?;
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


    #[doc(hidden)]
    pub fn __add_syntax_str(&mut self, contents: impl AsRef<str>, path: impl AsRef<Path>) -> &mut CodeGenJob {
        self.jobs
            .push(CodeGenJob::from_str(contents, path));

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


    #[doc(hidden)]
    pub fn __codegen_tokenstream(self, debug: bool) -> Result<TokenStream, CodegenError> {
        let gens = self.jobs
            .into_iter()
            .map(|i| i.__codegen_tokenstream(debug))
            .collect::<Result<Vec<_>, _>>()?;


        Ok(quote!(
            #(#gens)*
        ))
    }
}

impl Default for CodegenManager {
    fn default() -> Self {
        Self::new()
    }
}

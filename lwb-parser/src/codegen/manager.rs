use crate::codegen::generate_file_headers::write_headers;
// use crate::codegen::generate_language;
use crate::codegen::error::CodegenError;
use crate::codegen::error::CodegenError::NoExtension;
use crate::codegen::generate_from_pairs::generate_from_pairs;
use crate::codegen::generate_misc::{generate_parser, generate_root};
use crate::codegen::generate_structs::generate_structs;
use crate::codegen::generate_trait_impls::generate_trait_impls;
use crate::codegen::FormattingFile;
use crate::config::toml::{find_config_path, read_config, ReadConfigError};
use crate::config::Config;
use crate::language::Language;
use crate::parser::syntax_file::{convert_syntax_file_ast, ParseError, SyntaxFile};
use crate::sources::source_file::SourceFile;
use proc_macro2::TokenStream;
use quote::quote;
use std::io::Write;
use std::path::{Path, PathBuf};
use crate::error::display_miette_error;

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

fn codegen_internal(
    source: SourceFile,
    config: Config,
    imports: &[&str],
) -> Result<Generated, CodegenError> {
    let ast = SyntaxFile::parse(&source)?;

    let serialized_parser = bincode::serialize(&ast)?;

    let legacy_ast = convert_syntax_file_ast::convert(ast)?; // TODO make peg parser use new ast

    let mut derives = vec!["Debug"];

    if config.syntax.serde {
        derives.extend(["Serialize", "Deserialize"]);
    }

    let structs = generate_structs(&legacy_ast, &derives, config.syntax.non_exhaustive)?;
    let from_pairs = generate_from_pairs(&legacy_ast, config.syntax.non_exhaustive)?;
    let impls = generate_trait_impls(&legacy_ast)?;
    let root = generate_root(
        imports,
        &derives,
        &config.syntax.import_location,
        config.syntax.non_exhaustive,
    )?;
    let parser = generate_parser(&serialized_parser)?;

    Ok(Generated {
        impls,
        structs,
        from_pairs,
        root,
        parser,
    })
}

#[doc(hidden)]
pub fn __codegen_tokenstream(
    source: SourceFile,
    config: Config,
    debug: bool,
) -> Result<TokenStream, CodegenError> {
    let Generated {
        impls,
        structs,
        from_pairs,
        root,
        parser,
    } = codegen_internal(source, config, &[])?;

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

fn unwrap<T>(r: Result<T, ReadConfigError>) -> T {
    match r {
        Ok(i) => i,
        Err(e) => {
            panic!("failed to read config: {e}")
        }
    }
}

pub struct Codegen {
    config: Config,
}

impl Codegen {
    pub fn try_new() -> Result<Self, ReadConfigError> {
        Self::try_with_config(find_config_path())
    }

    pub fn try_with_config(path: PathBuf) -> Result<Self, ReadConfigError> {
        println!("cargo:rerun-if-changed={:?}", path);
        Ok(Self::with_config_struct(read_config(path)?))
    }

    pub fn with_config_struct(config: Config) -> Self {
        Self { config }
    }

    pub fn new() -> Self {
        unwrap(Self::try_new())
    }

    pub fn with_config(path: PathBuf) -> Self {
        unwrap(Self::try_with_config(path))
    }

    pub fn codegen(self) {
        if let Err(e) = self.try_codegen() {
            match e {
                CodegenError::ParseError(ParseError::PEG(errs)) => {
                    for e in errs {
                        eprintln!("{}", display_miette_error(&e));
                    }
                    panic!("failed to generate ast")
                }
                e => panic!("failed to generate ast: {e}")
            }
        }
    }

    pub fn try_codegen(self) -> Result<(), CodegenError> {
        let mut files = create_module_files(
            &self.config.syntax.destination,
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

        let write_serialized_ast = self.config.syntax.write_serialized_ast;

        let Generated {
            impls,
            structs,
            from_pairs,
            root,
            parser,
        } = codegen_internal(
            SourceFile::open(&self.config.syntax.definition)?,
            self.config,
            &["ast", "from_pairs", "ast_impls", "parser"],
        )?;

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

//! Analyse statique des modules Kastel sans les exécuter.

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::{
    error::compile_error::CompileError,
    frontend::{lexer::lexer::Lexer, parser::Parser},
    module::resolver::{ImportResolution, ModuleResolver},
};

use super::{type_checker::{TypeCheckContext, TypeChecker}, types::Type};

#[derive(Debug, Clone)]
pub struct ModuleTypeInterface {
    pub path: PathBuf,
    pub exports: HashMap<String, Type>,
}

#[derive(Debug, Clone)]
pub enum ImportedType {
    Module(PathBuf),
    Export(Type),
}

struct ModuleTypeLoaderState {
    cache: HashMap<PathBuf, Rc<ModuleTypeInterface>>,
    loading: HashSet<PathBuf>,
}

#[derive(Clone)]
pub struct ModuleTypeLoader {
    state: Rc<RefCell<ModuleTypeLoaderState>>,
    resolver: ModuleResolver,
}

impl ModuleTypeLoader {
    pub fn new(resolver: ModuleResolver) -> Self {
        Self {
            state: Rc::new(RefCell::new(ModuleTypeLoaderState {
                cache: HashMap::new(),
                loading: HashSet::new(),
            })),
            resolver,
        }
    }

    pub fn interface(&self, path: &Path) -> Result<Rc<ModuleTypeInterface>, CompileError> {
        let path = canonicalize_path(path)?;

        {
            let state = self.state.borrow();
            if let Some(interface) = state.cache.get(&path) {
                return Ok(Rc::clone(interface));
            }
            if state.loading.contains(&path) {
                return Err(CompileError::CircularImport(path.display().to_string()));
            }
        }

        self.state.borrow_mut().loading.insert(path.clone());

        let result = self.analyze_uncached(&path);

        {
            let mut state = self.state.borrow_mut();
            state.loading.remove(&path);
            if let Ok(interface) = &result {
                state.cache.insert(path.clone(), Rc::clone(interface));
            }
        }

        result
    }

    fn analyze_uncached(&self, path: &Path) -> Result<Rc<ModuleTypeInterface>, CompileError> {
        let source = fs::read_to_string(path).map_err(|error| CompileError::ModuleReadError {
            path: path.display().to_string(),
            message: error.to_string(),
        })?;

        let tokens = Lexer::new(source.clone())
            .scan_token()
            .map_err(|errors| CompileError::ModuleLexerErrors {
                path: path.display().to_string(),
                source: source.clone(),
                errors,
            })?;

        let mut parser = Parser::new(tokens);
        let statements = parser.parse().map_err(|errors| CompileError::ModuleParserErrors {
            path: path.display().to_string(),
            source: source.clone(),
            errors,
        })?;

        let context = TypeCheckContext::new(path.to_path_buf(), Rc::new(self.clone()));
        TypeChecker::analyze_module(&statements, context)
            .map(|exports| {
                Rc::new(ModuleTypeInterface {
                    path: path.to_path_buf(),
                    exports,
                })
            })
            .map_err(|error| match error {
                CompileError::ModuleCompileError { .. }
                | CompileError::ModuleParserErrors { .. }
                | CompileError::ModuleLexerErrors { .. }
                | CompileError::ModuleReadError { .. }
                | CompileError::ModuleNotFound(_)
                | CompileError::ModuleInvalidPath(_)
                | CompileError::CircularImport(_) => error,
                other => CompileError::ModuleCompileError {
                    path: path.display().to_string(),
                    module_source: source.clone(),
                    error: Box::new(other),
                },
            })
    }

    pub fn resolve_import(
        &self,
        current_file: &Path,
        parts: &[String],
    ) -> Result<ImportedType, CompileError> {
        match self.resolver.resolve_import(current_file, parts)? {
            ImportResolution::Module(path) => {
                Ok(ImportedType::Module(canonicalize_path(&path)?))
            }

            ImportResolution::Export { module, name } => {
                let interface = self.interface(&module)?;

                let ty = interface.exports.get(&name).cloned().ok_or_else(|| {
                    CompileError::ExportNotFound {
                        module: parts[..parts.len() - 1].join("."),
                        name: name.clone(),
                    }
                })?;

                Ok(ImportedType::Export(ty))
            }
        }
    }

    pub fn resolver(&self) -> &ModuleResolver {
        &self.resolver
    }
}

fn canonicalize_path(path: &Path) -> Result<PathBuf, CompileError> {
    path.canonicalize().map_err(|error| CompileError::ModuleReadError {
        path: path.display().to_string(),
        message: error.to_string(),
    })
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        compiler::type_checker::{TypeCheckContext, TypeChecker},
        frontend::{lexer::lexer::Lexer, parser::Parser},
    };
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(name);
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn parse(source: &str) -> Vec<crate::frontend::ast::Statement> {
        let tokens = Lexer::new(source.to_string()).scan_token().unwrap();
        Parser::new(tokens).parse().unwrap()
    }

    #[test]
    fn std_math_exports_keep_native_signature_and_infer_kastel_function() {
        let root = temp_dir("kastel_typecheck_std_math_test");
        let project = root.join("project");
        let std = root.join("std");
        fs::create_dir_all(&project).unwrap();
        fs::create_dir_all(&std).unwrap();
        fs::write(
            std.join("math.ks"),
            r#"
export const PI = 3.141592653589793;
export const sin = sin;
export func to_radians(degrees) {
    return degrees * PI / 180;
}
"#,
        )
        .unwrap();
        let main = project.join("main.ks");
        fs::write(&main, "").unwrap();

        let resolver = ModuleResolver::new(project).with_std_root(std);
        let loader = Rc::new(ModuleTypeLoader::new(resolver));

        let invalid = parse(
            r#"
import std.math
let a: int = math.sin(math.to_radians(90))
"#,
        );
        let invalid_result = TypeChecker::check_with_context(
            &invalid,
            TypeCheckContext::new(main.clone(), Rc::clone(&loader)),
        );
        assert!(invalid_result.is_err(), "float -> int must be rejected");

        let valid = parse(
            r#"
import std.math
let a: float = math.sin(math.to_radians(90))
"#,
        );
        let valid_result = TypeChecker::check_with_context(
            &valid,
            TypeCheckContext::new(main, loader),
        );
        assert!(valid_result.is_ok(), "float annotation must be accepted");

        let _ = fs::remove_dir_all(root);
    }
}

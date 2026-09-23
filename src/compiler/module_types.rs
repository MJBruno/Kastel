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

use super::{
    type_checker::{ClassInfo, TypeCheckContext, TypeChecker},
    types::Type,
};

#[derive(Debug, Clone)]
pub struct ModuleTypeInterface {
    pub path: PathBuf,
    pub exports: HashMap<String, Type>,

    /// Détail des classes et interfaces du module (constructeurs, méthodes
    /// surchargées, champs typés, membres privés), par nom.
    pub(crate) classes: HashMap<String, ClassInfo>,

    /// Alias de type EXPORTÉS (`export type Person = { ... };`), déjà
    /// résolus (sans référence aux noms locaux du module qui les déclare).
    pub(crate) type_aliases: HashMap<String, Type>,
}

#[derive(Debug, Clone)]
pub enum ImportedType {
    Module(PathBuf),

    /// Un export du module `interface`, sous le nom `name` (que le binding
    /// local peut renommer : `from m import Personne as P`).
    Export {
        ty: Type,
        name: String,
        interface: Rc<ModuleTypeInterface>,
    },

    /// Un alias de type exporté (`export type Person = { ... };`) : aucune
    /// valeur à l'exécution, seulement un type déjà résolu.
    TypeAlias {
        resolved: Type,
    },
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
            .map(|(exports, classes, type_aliases)| {
                Rc::new(ModuleTypeInterface {
                    path: path.to_path_buf(),
                    exports,
                    classes,
                    type_aliases,
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

                if let Some(ty) = interface.exports.get(&name).cloned() {
                    return Ok(ImportedType::Export {
                        ty,
                        name,
                        interface,
                    });
                }

                if let Some(resolved) = interface.type_aliases.get(&name).cloned() {
                    return Ok(ImportedType::TypeAlias { resolved });
                }

                Err(CompileError::ExportNotFound {
                    module: parts[..parts.len() - 1].join("."),
                    name: name.clone(),
                })
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

    #[test]
    fn exported_type_aliases_are_usable_from_another_module() {
        let root = temp_dir("kastel_typecheck_exported_alias_test");
        let project = root.join("project");
        fs::create_dir_all(&project).unwrap();
        fs::write(
            project.join("shapes.ks"),
            r#"
export type Point = { x: int, y: int };
export type Number = int | float;

// Alias NON exporté : ne doit PAS être visible depuis un autre module.
type Internal = str;

export func origin() -> Point {
    return { x: 0, y: 0 };
}
"#,
        )
        .unwrap();

        let main = project.join("main.ks");
        fs::write(&main, "").unwrap();

        let resolver = ModuleResolver::new(project);
        let loader = Rc::new(ModuleTypeLoader::new(resolver));

        let check = |source: &str| {
            TypeChecker::check_with_context(
                &parse(source),
                TypeCheckContext::new(main.clone(), Rc::clone(&loader)),
            )
        };

        // `from m import X;` pour un alias de type.
        assert!(
            check(
                r#"
from shapes import Point;
let p: Point = { x: 1, y: 2 };
let n: int = p.x;
"#
            )
            .is_ok()
        );
        assert!(
            check(
                r#"
from shapes import Point;
let p: Point = { x: 1 };
"#
            )
            .is_err(),
            "un champ manquant doit être refusé"
        );

        // `import m.X;` pour un alias de type — comme pour une classe.
        assert!(
            check(
                r#"
import shapes.Point;
let p: Point = { x: 1, y: 2 };
"#
            )
            .is_ok()
        );

        // Alias ET fonction du même module ensemble ; le type de retour
        // de `origin()` est bien `Point`.
        assert!(
            check(
                r#"
from shapes import Point, origin;
let p: Point = origin();
let x: int = p.x;
"#
            )
            .is_ok()
        );

        // `from m import *;` importe aussi les alias.
        assert!(
            check(
                r#"
from shapes import *;
let n: Number = 1;
let p: Point = { x: 1, y: 2 };
"#
            )
            .is_ok()
        );

        // Union importée : mêmes règles qu'un union local.
        assert!(
            check(
                r#"
from shapes import Number;
let n: Number = "x";
"#
            )
            .is_err()
        );

        // Alias NON exporté : introuvable.
        assert!(
            check("from shapes import Internal;").is_err(),
            "un alias non exporté ne doit pas être importable"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn union_typed_function_parameters_are_checked_across_modules() {
        let root = temp_dir("kastel_typecheck_union_param_test");
        let project = root.join("project");
        fs::create_dir_all(&project).unwrap();
        fs::write(
            project.join("mathx.ks"),
            r#"
export type Number = int | float;

// Union LITTÉRALE (pas d'alias) dans la signature.
export func classify(x: int | str) -> str {
    return "ok";
}

// Union VIA UN ALIAS déclaré APRÈS cette fonction : couvre exactement la
// régression corrigée (`register_aliases` devait déballer `export`).
export func half(n: Number) -> float {
    return n / 2;
}

export type Number2 = Number;
"#,
        )
        .unwrap();

        let main = project.join("main.ks");
        fs::write(&main, "").unwrap();

        let resolver = ModuleResolver::new(project);
        let loader = Rc::new(ModuleTypeLoader::new(resolver));

        let check = |source: &str| {
            TypeChecker::check_with_context(
                &parse(source),
                TypeCheckContext::new(main.clone(), Rc::clone(&loader)),
            )
        };

        // Union littérale : les deux membres passent, un troisième type non.
        assert!(check("from mathx import classify; let a: str = classify(1);").is_ok());
        assert!(check("from mathx import classify; let a: str = classify(\"x\");").is_ok());
        assert!(check("from mathx import classify; classify(1.5);").is_err());

        // Union via alias exporté, utilisée par une fonction déclarée AVANT
        // l'alias dans le fichier source.
        assert!(check("from mathx import half; let h: float = half(4);").is_ok());
        assert!(check("from mathx import half; let h: float = half(4.5);").is_ok());
        assert!(check("from mathx import half; half(\"x\");").is_err());

        // Alias qui référence lui-même un autre alias exporté (Number2 = Number).
        assert!(
            check("from mathx import Number2; let n: Number2 = 1; let m: Number2 = 2.5;")
                .is_ok()
        );
        assert!(check("from mathx import Number2; let n: Number2 = \"x\";").is_err());

        let _ = fs::remove_dir_all(root);
    }
}


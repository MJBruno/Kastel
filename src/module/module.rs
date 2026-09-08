use std::{
    cell::RefCell,
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::error::compile_error::CompileError;
use crate::frontend::lexer::Lexer;
use crate::frontend::parser::Parser;
use crate::runtime::native::execute_native;
use crate::vm::machine::VirtualMachine;
use crate::{compiler::compiler::Compiler, runtime::value::Value};

#[derive(Debug, PartialEq)]
pub struct ModuleInstance {
    pub name: String,
    pub path: PathBuf,
    pub exports: HashMap<String, Value>,
}
#[allow(dead_code)]
impl ModuleInstance {
    pub fn new(name: String, path: PathBuf) -> Self {
        Self {
            name,
            path,
            exports: HashMap::new(),
        }
    }

    pub fn get_export(&self, name: &str) -> Option<&Value> {
        self.exports.get(name)
    }

    pub fn export(&mut self, name: String, value: Value) -> Result<(), CompileError> {
        if self.exports.contains_key(&name) {
            return Err(CompileError::DuplicateExport(name));
        }

        self.exports.insert(name, value);

        Ok(())
    }
}

#[derive(Clone)]
pub struct ModuleLoader {
    state: Rc<RefCell<ModuleLoaderState>>,
}

struct ModuleLoaderState {
    cache: HashMap<PathBuf, Rc<ModuleInstance>>,
    loading: Vec<PathBuf>,
}
#[allow(dead_code)]
impl ModuleLoader {
    pub fn new() -> Self {
        Self {
            state: Rc::new(RefCell::new(ModuleLoaderState {
                cache: HashMap::new(),
                loading: Vec::new(),
            })),
        }
    }

    pub fn resolve(&self, current_file: &Path, parts: &[String]) -> Result<PathBuf, CompileError> {
        let parent = current_file
            .parent()
            .ok_or_else(|| CompileError::ModuleInvalidPath(current_file.display().to_string()))?;

        let mut path = parent.to_path_buf();

        for part in parts {
            path.push(part);
        }

        // ⚠️ Adapte "ks" à l'extension réelle de tes fichiers source Kastel
        // si ce n'est pas celle-ci (ex. "kastel", "kst"...).
        path.set_extension("ks");

        if !path.exists() {
            return Err(CompileError::ModuleNotFound(path.display().to_string()));
        }

        path.canonicalize()
            .map_err(|error| CompileError::ModuleReadError {
                path: path.display().to_string(),
                message: error.to_string(),
            })
    }

    pub fn load(&mut self, path: PathBuf) -> Result<Rc<ModuleInstance>, CompileError> {
        let path = path
            .canonicalize()
            .map_err(|error| CompileError::ModuleReadError {
                path: path.display().to_string(),
                message: error.to_string(),
            })?;

        {
            let state = self.state.borrow();

            if let Some(module) = state.cache.get(&path) {
                return Ok(Rc::clone(module));
            }
        }

        {
            let mut state = self.state.borrow_mut();

            if let Some(index) = state.loading.iter().position(|p| p == &path) {
                let mut cycle = state.loading[index..]
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>();

                cycle.push(path.display().to_string());

                return Err(CompileError::CircularImport(cycle.join(" -> ")));
            }

            state.loading.push(path.clone());
        }

        let result = self.load_uncached(&path);

        {
            let mut state = self.state.borrow_mut();

            let last = state.loading.pop();

            debug_assert_eq!(last.as_ref(), Some(&path));
        }

        result
    }

    fn load_uncached(&mut self, path: &Path) -> Result<Rc<ModuleInstance>, CompileError> {
        // ------------------------------------------------------------
        // 1. Lire le fichier
        // ------------------------------------------------------------
        let source = fs::read_to_string(path).map_err(|error| CompileError::ModuleReadError {
            path: path.display().to_string(),
            message: error.to_string(),
        })?;

        // ------------------------------------------------------------
        // 2. Lexer
        // ------------------------------------------------------------
        let mut lexer = Lexer::new(source);

        let tokens = lexer
            .scan_token()
            .map_err(CompileError::ModuleLexerErrors)?;
        // ------------------------------------------------------------
        // 3. Parser
        // ------------------------------------------------------------
        let mut parser = Parser::new(tokens);

        let statements = parser.parse().map_err(CompileError::ModuleParserErrors)?;
        // ------------------------------------------------------------
        // 4. Compiler
        // ------------------------------------------------------------
        let mut compiler = Compiler::new();

        execute_native(&mut compiler);

        let (function, exports) = compiler.compile_module(&statements).map_err(|error| {
            CompileError::ModuleCompileError {
                path: path.display().to_string(),
                source: Box::new(error),
            }
        })?;

        let function = Rc::new(function);

        // ------------------------------------------------------------
        // 5. Exécuter le module dans une VM isolée
        // ------------------------------------------------------------
        let module_loader = self.clone();

        let values = VirtualMachine::execute_module(
            Rc::clone(&function),
            &exports,
            path.to_path_buf(),
            module_loader,
        )
        .map_err(|error| CompileError::ModuleRuntimeError {
            path: path.display().to_string(),
            source: error,
        })?;
        // ------------------------------------------------------------
        // 6. Nom du module
        // ------------------------------------------------------------
        let name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("<module>")
            .to_string();

        // ------------------------------------------------------------
        // 7. Construire l'instance
        // ------------------------------------------------------------
        let mut module = ModuleInstance::new(name, path.to_path_buf());

        // ------------------------------------------------------------
        // 8. Ajouter uniquement les exports
        // ------------------------------------------------------------
        for (name, value) in values {
            module.export(name, value)?;
        }

        // ------------------------------------------------------------
        // 9. Mettre en cache
        // ------------------------------------------------------------
        let module = Rc::new(module);

        self.state
            .borrow_mut()
            .cache
            .insert(path.to_path_buf(), Rc::clone(&module));

        Ok(module)
    }
    pub fn load_from(
        &mut self,
        current_file: &Path,
        parts: &[String],
    ) -> Result<Rc<ModuleInstance>, CompileError> {
        let path = self.resolve(current_file, parts)?;

        self.load(path)
    }
}

use std::{
    cell::RefCell,
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
};


use crate::frontend::{lexer::lexer::Lexer, parser::Parser};
use crate::module::resolver::ModuleResolver;
use crate::vm::machine::VirtualMachine;
use crate::{compiler::{compiler::Compiler, module_types::ModuleTypeLoader, type_checker::TypeCheckContext}, runtime::value::Value};
use crate::{error::compile_error::CompileError, stdlib::execute_native};

#[derive(Debug)]
pub struct ModuleInstance {
    pub name: String,
    pub path: PathBuf,

    /// Environnement global propre au module.
    ///
    /// Les closures créées pendant l'exécution du module conservent une
    /// `Weak` vers cette table afin de pouvoir résoudre leurs globals au
    /// moment de l'appel, même après la fin de `execute_module()`.
    pub globals: Rc<RefCell<HashMap<String, Value>>>,

    pub exports: HashMap<String, Value>,
}

impl PartialEq for ModuleInstance {
    fn eq(&self, other: &Self) -> bool {
        // Un module est identifié par son chemin résolu. Les globals/exports
        // ne font volontairement pas partie de cette égalité : ils peuvent
        // contenir des closures, classes ou modules se référant entre eux.
        self.path == other.path
    }
}
#[allow(dead_code)]
impl ModuleInstance {
    pub fn new(name: String, path: PathBuf) -> Self {
        Self {
            name,
            path,
            globals: Rc::new(RefCell::new(HashMap::new())),
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
    resolver: Rc<ModuleResolver>,
    type_loader: Rc<ModuleTypeLoader>,
}

struct ModuleLoaderState {
    cache: HashMap<PathBuf, Rc<ModuleInstance>>,
    loading: Vec<PathBuf>,
}
#[allow(dead_code)]
impl ModuleLoader {
    /// `project_root` est le répertoire du fichier d'entrée du
    /// programme (voir `ModuleResolver::new`) : c'est lui qui sert de
    /// repli pour les imports « absolus » relatifs au projet, et qui
    /// détermine où se trouve `std/` par défaut.
    pub fn new(project_root: PathBuf) -> Self {
        Self::with_resolver(ModuleResolver::new(project_root))
    }

    pub fn with_resolver(resolver: ModuleResolver) -> Self {
        Self {
            state: Rc::new(RefCell::new(ModuleLoaderState {
                cache: HashMap::new(),
                loading: Vec::new(),
            })),
            resolver: Rc::new(resolver.clone()),
            type_loader: Rc::new(ModuleTypeLoader::new(resolver)),
        }
    }

    pub fn resolver(&self) -> &ModuleResolver {
        &self.resolver
    }

    pub fn loaded_modules(&self) -> Vec<Rc<ModuleInstance>> {
        self.state.borrow().cache.values().cloned().collect()
    }

    pub fn resolve(&self, current_file: &Path, parts: &[String]) -> Result<PathBuf, CompileError> {
        self.resolver.resolve(current_file, parts)
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
        let mut lexer = Lexer::new(source.clone());

        let tokens = lexer.scan_token().map_err(|errors| CompileError::ModuleLexerErrors {
            path: path.display().to_string(),
            source: source.clone(),
            errors,
        })?;
        // ------------------------------------------------------------
        // 3. Parser
        // ------------------------------------------------------------
        let mut parser = Parser::new(tokens);

        let statements = parser
            .parse()
            .map_err(|errors| CompileError::ModuleParserErrors {
                path: path.display().to_string(),
                source: source.clone(),
                errors,
            })?;
        // ------------------------------------------------------------
        // 4. Compiler
        // ------------------------------------------------------------
        let mut compiler = Compiler::new();

        execute_native(&mut compiler);

        let context = TypeCheckContext::new(
            path.to_path_buf(),
            Rc::clone(&self.type_loader),
        );

        let (function, exports) = compiler
            .compile_module_with_context(&statements, context)
            .map_err(|error| {
            CompileError::ModuleCompileError {
                path: path.display().to_string(),
                module_source: source.clone(),
                error: Box::new(error),
            }
        })?;

        let function = Rc::new(function);

        // ------------------------------------------------------------
        // 5. Construire l'instance et son environnement global AVANT
        //    l'exécution : les closures créées par le module doivent
        //    conserver ce même environnement après le retour de la VM.
        // ------------------------------------------------------------
        let name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("<module>")
            .to_string();

        let mut module = ModuleInstance::new(name, path.to_path_buf());

        // ------------------------------------------------------------
        // 6. Exécuter le module dans une VM isolée partageant l'env.
        // ------------------------------------------------------------
        let module_loader = self.clone();
        let module_globals = Rc::clone(&module.globals);

        let values = VirtualMachine::execute_module(
            Rc::clone(&function),
            &exports,
            path.to_path_buf(),
            module_loader,
            module_globals,
        )
        .map_err(|error| CompileError::ModuleRuntimeError {
            path: path.display().to_string(),
            module_source: source.clone(),
            source: error,
        })?;
        // ------------------------------------------------------------
        // 7. Ajouter uniquement les exports
        // ------------------------------------------------------------
        for (name, value) in values {
            module.export(name, value)?;
        }

        // ------------------------------------------------------------
        // 8. Mettre en cache
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

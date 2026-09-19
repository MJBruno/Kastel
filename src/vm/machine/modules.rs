use std::{path::Path, rc::Rc};

use super::VirtualMachine;
use crate::error::runtime_error::RuntimeError;
use crate::module::resolver::ImportResolution;
use crate::runtime::object::Object;
use crate::runtime::value::Value;

impl VirtualMachine {
    pub(crate) fn import_module(&mut self) -> Result<(), RuntimeError> {
        let constant = self.read_byte()?;
        let value = self.read_constant(constant)?;
        let module_name = value.as_string_value().ok_or(RuntimeError::TypeError)?;

        let current_file = self.module_path.clone().ok_or_else(|| {
            RuntimeError::ModuleError("Cannot import module without a source path".to_string())
        })?;

        let parts = module_name
            .split('.')
            .map(str::to_string)
            .collect::<Vec<_>>();

        if parts.is_empty() {
            return Err(RuntimeError::ModuleError("Invalid module name".to_string()));
        }

        let resolved = self.resolve_import_value(&current_file, &parts, &module_name)?;

        self.push(resolved);

        Ok(())
    }

    /// Résout un `import a.b.c;` en essayant, dans l'ordre :
    ///
    ///   1. `a.b.c` comme chemin de sous-module complet (ex.
    ///      `import std.math;` -> la valeur renvoyée est le MODULE
    ///      `std/math.ks` lui-même, ce qui permet ensuite
    ///      `math.sqrt(2.0)`) ;
    ///   2. si (1) échoue ET qu'il y a au moins 2 segments, `a.b`
    ///      comme module et `c` comme export à en extraire (ex.
    ///      `import shapes.Circle;` -> la valeur renvoyée est
    ///      directement la CLASSE `Circle`, ce qui permet
    ///      `new Circle()` sans qualifier par le nom du module).
    ///
    /// Dans les deux cas, `compile_import` a déjà décidé de lier le
    /// DERNIER segment du chemin (`math`, `Circle`) — c'est donc bien
    /// ce choix, fait ici à l'exécution plutôt qu'à la compilation
    /// (qui n'a pas accès au système de fichiers), qui détermine si
    /// la valeur obtenue est un module ou l'un de ses exports.
    fn resolve_import_value(
        &mut self,
        current_file: &Path,
        parts: &[String],
        module_name: &str,
    ) -> Result<Value, RuntimeError> {
        match self
            .module_loader
            .resolve_import(current_file, parts)
            .map_err(|error| RuntimeError::ModuleError(error.to_string()))?
        {
            ImportResolution::Module(path) => {
                let module = self
                    .module_loader
                    .load(path)
                    .map_err(|error| RuntimeError::ModuleError(error.to_string()))?;

                Ok(Value::new_module(module))
            }

            ImportResolution::Export { module, name } => {
                let module = self
                    .module_loader
                    .load(module)
                    .map_err(|error| RuntimeError::ModuleError(error.to_string()))?;

                module.get_export(&name).cloned().ok_or_else(|| {
                    RuntimeError::ModuleError(format!(
                        "le module '{}' n'exporte pas '{}'",
                        module_name, name
                    ))
                })
            }
        }
    }

    pub(crate) fn import_all(&mut self) -> Result<(), RuntimeError> {
        let constant = self.read_byte()?;
        let value = self.read_constant(constant)?;

        let module_name = value.as_string_value().ok_or(RuntimeError::TypeError)?;

        let current_file = self.module_path.clone().ok_or_else(|| {
            RuntimeError::ModuleError("Cannot import module without a source path".to_string())
        })?;

        let parts = module_name
            .split('.')
            .map(str::to_string)
            .collect::<Vec<_>>();

        if parts.is_empty() {
            return Err(RuntimeError::ModuleError("Invalid module name".to_string()));
        }

        let module = self
            .module_loader
            .load_from(&current_file, &parts)
            .map_err(|error| RuntimeError::ModuleError(error.to_string()))?;

        let module_object = Value::new_module(module);

        let exports = match &module_object {
            Value::Object(handle) => {
                let object = handle.borrow();

                match &*object {
                    Object::Module(module) => module
                        .exports
                        .iter()
                        .map(|(name, value)| (name.clone(), value.clone()))
                        .collect::<Vec<_>>(),

                    _ => {
                        return Err(RuntimeError::TypeError);
                    }
                }
            }

            _ => {
                return Err(RuntimeError::TypeError);
            }
        };

        let globals = self
            .frames
            .last()
            .and_then(|frame| super::bytecode::frame_closure(&frame.closure).global_env.upgrade())
            .unwrap_or_else(|| Rc::clone(&self.globals));

        let mut globals = globals.borrow_mut();

        for (name, value) in exports {
            if globals.contains_key(&name) {
                return Err(RuntimeError::ModuleError(format!(
                    "Cannot import '{}' from module '{}': \
                         a global with the same name already exists",
                    name, module_name
                )));
            }

            globals.insert(name, value);
        }

        Ok(())
    }
}

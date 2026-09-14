use super::VirtualMachine;
use crate::error::runtime_error::RuntimeError;
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

        let module = self
            .module_loader
            .load_from(&current_file, &parts)
            .map_err(|error| RuntimeError::ModuleError(error.to_string()))?;

        self.push(Value::new_module(module));

        Ok(())
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

        for (name, value) in exports {
            if self.globals.contains_key(&name) {
                return Err(RuntimeError::ModuleError(format!(
                    "Cannot import '{}' from module '{}': \
                         a global with the same name already exists",
                    name, module_name
                )));
            }

            self.globals.insert(name, value);
        }

        Ok(())
    }
}

use std::collections::HashMap;

use crate::{
    compiler::compiler::Compiler, error::runtime_error::RuntimeError, runtime::value::Value,
};

/// Construit un mutex coopératif.
/// Le verrouillage/déverrouillage exige une tâche Kastel active.
pub fn native_mutex(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 0,
            found: args.len(),
        });
    }

    Ok(Value::new_mutex())
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert("mutex".into(), Value::NativeFunction(native_mutex));
}

pub fn register_compiler(compiler: &mut Compiler) {
    let _ = compiler.define_native("mutex");
}

use std::collections::HashMap;

use crate::{
    error::runtime_error::RuntimeError, runtime::value::Value,
};

/// Construit un compteur coopératif de tâches.
pub fn native_wait_group(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 0,
            found: args.len(),
        });
    }

    Ok(Value::new_wait_group())
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert(
        "wait_group".into(),
        Value::NativeFunction(native_wait_group),
    );
}


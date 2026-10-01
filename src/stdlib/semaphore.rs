use std::collections::HashMap;

use crate::{
    error::runtime_error::RuntimeError, runtime::value::Value,
};

/// Construit un sémaphore coopératif avec `capacity` permis.
pub fn native_semaphore(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let capacity = match args[0] {
        Value::Integer(value) if value > 0 => value as usize,
        Value::Integer(_) => return Err(RuntimeError::SemaphoreNonPositive),
        _ => return Err(RuntimeError::TypeError),
    };

    Ok(Value::new_semaphore(capacity))
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert("semaphore".into(), Value::NativeFunction(native_semaphore));
}


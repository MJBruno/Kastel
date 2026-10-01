use std::collections::HashMap;

use crate::{
    error::runtime_error::RuntimeError, runtime::object::Object,
    runtime::value::Value,
};

/// Construit une variable de condition associée à un `Mutex`.
pub fn native_condvar(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let mutex = match &args[0] {
        Value::Object(handle) => {
            let object = handle.borrow();
            match &*object {
                Object::Mutex(mutex) => mutex.clone(),
                _ => return Err(RuntimeError::TypeError),
            }
        }
        _ => return Err(RuntimeError::TypeError),
    };

    Ok(Value::new_condvar(mutex))
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert("condvar".into(), Value::NativeFunction(native_condvar));
}


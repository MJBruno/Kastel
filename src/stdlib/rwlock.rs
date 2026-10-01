use std::collections::HashMap;

use crate::{
    error::runtime_error::RuntimeError, runtime::value::Value,
};

/// Construit un verrou lecture/écriture coopératif.
/// Les acquisitions bloquantes respectent l'ordre FIFO des demandes ; les
/// lecteurs placés consécutivement avant un écrivain peuvent être réveillés ensemble.
pub fn native_rwlock(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 0,
            found: args.len(),
        });
    }

    Ok(Value::new_rwlock())
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert("rwlock".into(), Value::NativeFunction(native_rwlock));
}


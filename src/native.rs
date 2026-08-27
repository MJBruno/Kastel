use crate::error::RuntimeError;
use crate::value::Value;

use std::time::{SystemTime, UNIX_EPOCH};

pub type NativeFn = fn(&[Value]) -> Result<Value, RuntimeError>;

pub fn native_clock(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 0,
            found: args.len(),
        });
    }

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| RuntimeError::TypeError)?;

    Ok(Value::Number(duration.as_secs_f64() ))
}

pub fn native_add(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let a = match &args[0] {
        Value::Number(value) => *value,
        _ => return Err(RuntimeError::TypeError),
    };

    let b = match &args[1] {
        Value::Number(value) => *value,
        _ => return Err(RuntimeError::TypeError),
    };

    Ok(Value::Number(a + b))
}

pub fn register_natives(
    globals: &mut std::collections::HashMap<String, Value>,
) {
    globals.insert(
        "clock".to_string(),
        Value::NativeFunction {
            function: native_clock,
            arity: 0,
        },
    );

    globals.insert(
        "native_add".to_string(),
        Value::NativeFunction {
            function: native_add,
            arity: 2,
        },
    );
}
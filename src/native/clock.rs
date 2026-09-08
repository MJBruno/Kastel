
// ================================================================
// src/native/clock.rs
// ================================================================

use crate::{
    compiler::compiler::Compiler,
    error::runtime_error::RuntimeError,
    runtime::value::Value,
};

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

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

    Ok(Value::Float(duration.as_secs_f64()))
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert("clock".to_string(), Value::NativeFunction(native_clock));
}

pub fn register_compiler(compiler: &mut Compiler) {
    let _ = compiler.define_native("clock");
}


use std::collections::HashMap;

use crate::{
    compiler::compiler::Compiler, error::runtime_error::RuntimeError, runtime::value::Value,
};

/// Construit un événement coopératif non signalé.
pub fn native_event(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 0,
            found: args.len(),
        });
    }

    Ok(Value::new_event())
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert("event".into(), Value::NativeFunction(native_event));
}

pub fn register_compiler(compiler: &mut Compiler) {
    let _ = compiler.define_native("event");
}

use std::collections::HashMap;

use crate::{
    compiler::compiler::Compiler,
    error::runtime_error::RuntimeError,
    runtime::value::Value,
};

/// Construit un canal coopératif non borné.
pub fn native_channel(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 0,
            found: args.len(),
        });
    }

    Ok(Value::new_channel())
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert("channel".into(), Value::NativeFunction(native_channel));
}

pub fn register_compiler(compiler: &mut Compiler) {
    let _ = compiler.define_native("channel");
}

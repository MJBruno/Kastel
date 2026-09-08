use std::collections::HashMap;

use crate::{
    compiler::compiler::Compiler, error::runtime_error::RuntimeError, runtime::value::Value,
};

pub fn native_inspect(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    Ok(Value::new_string(args[0].to_string()))
}

pub fn native_debug(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    eprintln!("[KASTEL DEBUG] {}", args[0]);

    Ok(args[0].clone())
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert("inspect".to_string(), Value::NativeFunction(native_inspect));

    globals.insert("debug".to_string(), Value::NativeFunction(native_debug));
}

pub fn register_compiler(compiler: &mut Compiler) {
    let _ = compiler.define_native("inspect");
    let _ = compiler.define_native("debug");
}

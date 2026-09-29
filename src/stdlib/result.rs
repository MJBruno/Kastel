//! Runtime support for `Result<T, E>`.

use crate::{
    compiler::compiler::Compiler, error::runtime_error::RuntimeError, runtime::value::Value,
};
use std::collections::HashMap;

pub fn native_ok(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }
    Ok(Value::new_ok(args[0].clone()))
}

pub fn native_err(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }
    Ok(Value::new_err(args[0].clone()))
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert("Ok".into(), Value::NativeFunction(native_ok));
    globals.insert("Err".into(), Value::NativeFunction(native_err));
}

pub fn register_compiler(compiler: &mut Compiler) {
    let _ = compiler.define_native("Ok");
    let _ = compiler.define_native("Err");
}

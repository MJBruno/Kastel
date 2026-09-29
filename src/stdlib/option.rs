//! Runtime support for `Option<T>`.

use crate::{
    compiler::compiler::Compiler, error::runtime_error::RuntimeError, runtime::value::Value,
};
use std::collections::HashMap;

pub fn native_some(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }
    Ok(Value::new_some(args[0].clone()))
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert("Some".into(), Value::NativeFunction(native_some));
}

pub fn register_compiler(compiler: &mut Compiler) {
    let _ = compiler.define_native("Some");
}

// ================================================================
// src/native/types.rs
// ================================================================

use crate::{
    compiler::compiler::Compiler,
    error::runtime_error::RuntimeError,
    runtime::object::Object,
    runtime::value::Value,
};

use std::collections::HashMap;

pub fn native_type(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let name = match &args[0] {
        Value::Integer(_) => "int",
        Value::Float(_) => "float",
        Value::Boolean(_) => "bool",
        Value::Nil => "nil",
        Value::Range { .. } => "range",
        Value::NativeFunction(_) => "function",

        Value::Object(handle) => match &*handle.borrow() {
            Object::String(_) => "string",
            Object::Array(_) => "array",
            Object::Dict(_) => "object",
            Object::Function(_) | Object::Closure(_) => "function",
            Object::Iterator(_) => "iterator",
            Object::Module(_) => "module",
        },
    };

    Ok(Value::new_string(name.to_string()))
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert("type".to_string(), Value::NativeFunction(native_type));
}

pub fn register_compiler(compiler: &mut Compiler) {
    let _ = compiler.define_native("type");
}

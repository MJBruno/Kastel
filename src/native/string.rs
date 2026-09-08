
// ================================================================
// src/native/string.rs
// ================================================================

use crate::{
    compiler::compiler::Compiler,
    error::runtime_error::RuntimeError,
    runtime::value::Value,
};

use std::collections::HashMap;

pub(crate) fn format_string(
    format: &str,
    args: &[Value],
) -> Result<String, RuntimeError> {
    let mut result = String::with_capacity(format.len());
    let mut chars = format.chars().peekable();
    let mut arg_index = 0;

    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'}') {
            chars.next();

            let value = args
                .get(arg_index)
                .ok_or(RuntimeError::WrongArgumentCount {
                    expected: arg_index + 1,
                    found: args.len(),
                })?;

            result.push_str(&value.to_string());

            arg_index += 1;
        } else {
            result.push(c);
        }
    }

    Ok(result)
}

pub fn native_format(args: &[Value]) -> Result<Value, RuntimeError> {
    let Some(first) = args.first() else {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: 0,
        });
    };

    let format = first
        .as_string_value()
        .ok_or(RuntimeError::TypeError)?;

    let formatted = format_string(&format, &args[1..])?;

    Ok(Value::new_string(formatted))
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert(
        "format".to_string(),
        Value::NativeFunction(native_format),
    );
}

pub fn register_compiler(compiler: &mut Compiler) {
    let _ = compiler.define_native("format");
}


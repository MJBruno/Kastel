
// ================================================================
// src/native/array.rs
// ================================================================

use crate::{
    compiler::compiler::Compiler,
    error::runtime_error::RuntimeError,
    runtime::value::Value,
};

use std::collections::HashMap;

fn expect_array_index(value: &Value) -> Result<usize, RuntimeError> {
    match value {
        Value::Integer(n) if *n >= 0 => Ok(*n as usize),

        Value::Float(n)
            if n.is_finite()
                && *n >= 0.0
                && n.fract() == 0.0
                && *n <= usize::MAX as f64 =>
        {
            Ok(*n as usize)
        }

        _ => Err(RuntimeError::TypeError),
    }
}

pub fn native_array_push(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let array = &args[0];
    let value = args[1].clone();

    let length = array.array_push(value)?;

    Ok(Value::Integer(length as i64))
}

pub fn native_array_pop(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    args[0].array_pop()
}

pub fn native_array_length(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let length = args[0].array_len()?;

    Ok(Value::Integer(length as i64))
}

pub fn native_array_insert(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 3,
            found: args.len(),
        });
    }

    let index = expect_array_index(&args[1])?;

    let length = args[0].array_insert(
        index,
        args[2].clone(),
    )?;

    Ok(Value::Integer(length as i64))
}

pub fn native_array_remove(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let index = expect_array_index(&args[1])?;

    args[0].array_remove(index)
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert(
        "push".to_string(),
        Value::NativeFunction(native_array_push),
    );

    globals.insert(
        "pop".to_string(),
        Value::NativeFunction(native_array_pop),
    );

    globals.insert(
        "length".to_string(),
        Value::NativeFunction(native_array_length),
    );

    globals.insert(
        "insert".to_string(),
        Value::NativeFunction(native_array_insert),
    );

    globals.insert(
        "remove".to_string(),
        Value::NativeFunction(native_array_remove),
    );
}

pub fn register_compiler(compiler: &mut Compiler) {
    let _ = compiler.define_native("push");
    let _ = compiler.define_native("pop");
    let _ = compiler.define_native("length");
    let _ = compiler.define_native("insert");
    let _ = compiler.define_native("remove");
}

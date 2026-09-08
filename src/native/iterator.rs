
// ================================================================
// src/native/iterator.rs
// ================================================================

use crate::{
    compiler::compiler::Compiler,
    error::runtime_error::RuntimeError,
    runtime::value::Value,
};

use std::collections::HashMap;

fn expect_integer(value: &Value) -> Result<i64, RuntimeError> {
    match value {
        Value::Integer(n) => Ok(*n),

        Value::Float(n) => {
            if !n.is_finite()
                || n.fract() != 0.0
                || *n < i64::MIN as f64
                || *n > i64::MAX as f64
            {
                return Err(RuntimeError::TypeError);
            }

            Ok(*n as i64)
        }

        _ => Err(RuntimeError::TypeError),
    }
}

pub fn native_range(args: &[Value]) -> Result<Value, RuntimeError> {
    let (start, stop, step) = match args.len() {
        1 => (
            0i64,
            expect_integer(&args[0])?,
            1i64,
        ),

        2 => (
            expect_integer(&args[0])?,
            expect_integer(&args[1])?,
            1i64,
        ),

        3 => (
            expect_integer(&args[0])?,
            expect_integer(&args[1])?,
            expect_integer(&args[2])?,
        ),

        found => {
            return Err(RuntimeError::WrongArgumentCount {
                expected: 3,
                found,
            });
        }
    };

    if step == 0 {
        return Err(RuntimeError::TypeError);
    }

    Ok(Value::new_range(
        start as f64,
        stop as f64,
        step as f64,
    ))
}

pub fn native_list(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    crate::runtime::iterator::drain_to_array(&args[0])
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert(
        "range".to_string(),
        Value::NativeFunction(native_range),
    );

    globals.insert(
        "list".to_string(),
        Value::NativeFunction(native_list),
    );
}

pub fn register_compiler(compiler: &mut Compiler) {
    let _ = compiler.define_native("range");
    let _ = compiler.define_native("list");
}


use std::collections::HashMap;

use crate::{
    compiler::compiler::Compiler, error::runtime_error::RuntimeError, runtime::value::Value,
};

fn expect_index(value: &Value) -> Result<usize, RuntimeError> {
    match value {
        Value::Integer(index) if *index >= 0 => {
            usize::try_from(*index).map_err(|_| RuntimeError::TypeError)
        }

        Value::Float(index) if index.is_finite() && *index >= 0.0 && index.fract() == 0.0 => {
            if *index > usize::MAX as f64 {
                return Err(RuntimeError::TypeError);
            }

            Ok(*index as usize)
        }

        _ => Err(RuntimeError::TypeError),
    }
}

pub fn native_push(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let length = args[0].array_push(args[1].clone())?;

    Ok(Value::Integer(length as i64))
}

pub fn native_pop(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    args[0].array_pop()
}

pub fn native_length(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    Ok(Value::Integer(args[0].array_len()? as i64))
}

pub fn native_insert(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 3,
            found: args.len(),
        });
    }

    let index = expect_index(&args[1])?;

    let length = args[0].array_insert(index, args[2].clone())?;

    Ok(Value::Integer(length as i64))
}

pub fn native_remove(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let index = expect_index(&args[1])?;

    args[0].array_remove(index)
}

pub fn native_contains(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    Ok(Value::Boolean(args[0].array_contains(&args[1])?))
}

pub fn native_clear(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    args[0].array_clear()?;

    Ok(Value::Nil)
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert("push".to_string(), Value::NativeFunction(native_push));

    globals.insert("pop".to_string(), Value::NativeFunction(native_pop));

    globals.insert("length".to_string(), Value::NativeFunction(native_length));

    globals.insert("insert".to_string(), Value::NativeFunction(native_insert));

    globals.insert("remove".to_string(), Value::NativeFunction(native_remove));

    globals.insert(
        "contains".to_string(),
        Value::NativeFunction(native_contains),
    );

    globals.insert("clear".to_string(), Value::NativeFunction(native_clear));
}

pub fn register_compiler(compiler: &mut Compiler) {
    let names = [
        "push", "pop", "length", "insert", "remove", "contains", "clear",
    ];

    for name in names {
        let _ = compiler.define_native(name);
    }
}

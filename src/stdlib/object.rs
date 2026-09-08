use std::collections::HashMap;

use crate::{
    compiler::compiler::Compiler,
    error::runtime_error::RuntimeError,
    runtime::object::Object,
    runtime::value::Value,
};

fn expect_object_fields(
    value: &Value,
) -> Result<Vec<(String, Value)>, RuntimeError> {
    match value {
        Value::Object(handle) => match &*handle.borrow() {
            Object::Dict(fields) => Ok(fields.clone()),
            _ => Err(RuntimeError::TypeError),
        },

        _ => Err(RuntimeError::TypeError),
    }
}

pub fn native_get(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let key = args[1]
        .as_string_value()
        .ok_or(RuntimeError::TypeError)?;

    args[0].object_get(&key)
}

pub fn native_set(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 3,
            found: args.len(),
        });
    }

    let key = args[1]
        .as_string_value()
        .ok_or(RuntimeError::TypeError)?;

    args[0].object_set(&key, args[2].clone())?;

    Ok(Value::Nil)
}

pub fn native_has(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let key = args[1]
        .as_string_value()
        .ok_or(RuntimeError::TypeError)?;

    let fields = expect_object_fields(&args[0])?;

    Ok(Value::Boolean(
        fields.iter().any(|(name, _)| name == &key),
    ))
}

pub fn native_keys(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let fields = expect_object_fields(&args[0])?;

    let keys = fields
        .into_iter()
        .map(|(key, _)| Value::new_string(key))
        .collect();

    Ok(Value::new_array(keys))
}

pub fn native_values(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let fields = expect_object_fields(&args[0])?;

    let values = fields
        .into_iter()
        .map(|(_, value)| value)
        .collect();

    Ok(Value::new_array(values))
}

pub fn native_object_length(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let fields = expect_object_fields(&args[0])?;

    Ok(Value::Integer(fields.len() as i64))
}

pub fn native_object(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 0,
            found: args.len(),
        });
    }

    Ok(Value::new_object(Vec::new()))
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert(
        "object".to_string(),
        Value::NativeFunction(native_object),
    );

    globals.insert(
        "get".to_string(),
        Value::NativeFunction(native_get),
    );

    globals.insert(
        "set".to_string(),
        Value::NativeFunction(native_set),
    );

    globals.insert(
        "has".to_string(),
        Value::NativeFunction(native_has),
    );

    globals.insert(
        "keys".to_string(),
        Value::NativeFunction(native_keys),
    );

    globals.insert(
        "values".to_string(),
        Value::NativeFunction(native_values),
    );

    globals.insert(
        "object_length".to_string(),
        Value::NativeFunction(native_object_length),
    );
}

pub fn register_compiler(compiler: &mut Compiler) {
    let names = [
        "object",
        "get",
        "set",
        "has",
        "keys",
        "values",
        "object_length",
    ];

    for name in names {
        let _ = compiler.define_native(name);
    }
}


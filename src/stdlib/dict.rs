use std::collections::HashMap;

use crate::{
    compiler::compiler::Compiler,
    error::runtime_error::RuntimeError,
    runtime::object::Object,
    runtime::value::Value,
};

fn expect_dict(value: &Value) -> Result<(), RuntimeError> {
    match value {
        Value::Object(handle) => match &*handle.borrow() {
            Object::Dict(_) => Ok(()),
            _ => Err(RuntimeError::TypeError),
        },

        _ => Err(RuntimeError::TypeError),
    }
}

pub fn native_dict(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 0,
            found: args.len(),
        });
    }

    Ok(Value::new_dict(Vec::new()))
}

pub fn native_dict_get(
    args: &[Value],
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let key = args[1]
        .as_string_value()
        .ok_or(RuntimeError::TypeError)?;

    args[0].dict_get(&key)
}

pub fn native_dict_set(
    args: &[Value],
) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 3,
            found: args.len(),
        });
    }

    let key = args[1]
        .as_string_value()
        .ok_or(RuntimeError::TypeError)?;

    args[0].dict_set(&key, args[2].clone())?;

    Ok(Value::Nil)
}

pub fn native_dict_has(
    args: &[Value],
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let key = args[1]
        .as_string_value()
        .ok_or(RuntimeError::TypeError)?;

    Ok(Value::Boolean(args[0].dict_contains(&key)?))
}

pub fn native_dict_remove(
    args: &[Value],
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let key = args[1]
        .as_string_value()
        .ok_or(RuntimeError::TypeError)?;

    args[0].dict_remove(&key)
}

pub fn native_dict_keys(
    args: &[Value],
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    args[0].dict_keys()
}

pub fn native_dict_values(
    args: &[Value],
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    args[0].dict_values()
}

pub fn native_dict_items(
    args: &[Value],
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    args[0].dict_items()
}

pub fn native_dict_length(
    args: &[Value],
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    expect_dict(&args[0])?;

    Ok(Value::Integer(
        args[0].dict_len()? as i64,
    ))
}

pub fn native_dict_clear(
    args: &[Value],
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    match &args[0] {
        Value::Object(handle) => {
            let mut object = handle.borrow_mut();

            match &mut *object {
                Object::Dict(entries) => {
                    entries.clear();
                    Ok(Value::Nil)
                }

                _ => Err(RuntimeError::TypeError),
            }
        }

        _ => Err(RuntimeError::TypeError),
    }
}

pub fn register(
    globals: &mut HashMap<String, Value>,
) {
    globals.insert(
        "dict".to_string(),
        Value::NativeFunction(native_dict),
    );

    globals.insert(
        "dict_get".to_string(),
        Value::NativeFunction(native_dict_get),
    );

    globals.insert(
        "dict_set".to_string(),
        Value::NativeFunction(native_dict_set),
    );

    globals.insert(
        "dict_has".to_string(),
        Value::NativeFunction(native_dict_has),
    );

    globals.insert(
        "dict_remove".to_string(),
        Value::NativeFunction(native_dict_remove),
    );

    globals.insert(
        "dict_keys".to_string(),
        Value::NativeFunction(native_dict_keys),
    );

    globals.insert(
        "dict_values".to_string(),
        Value::NativeFunction(native_dict_values),
    );

    globals.insert(
        "dict_items".to_string(),
        Value::NativeFunction(native_dict_items),
    );

    globals.insert(
        "dict_length".to_string(),
        Value::NativeFunction(native_dict_length),
    );

    globals.insert(
        "dict_clear".to_string(),
        Value::NativeFunction(native_dict_clear),
    );
}

pub fn register_compiler(
    compiler: &mut Compiler,
) {
    let names = [
        "dict",
        "dict_get",
        "dict_set",
        "dict_has",
        "dict_remove",
        "dict_keys",
        "dict_values",
        "dict_items",
        "dict_length",
        "dict_clear",
    ];

    for name in names {
        let _ = compiler.define_native(name);
    }
}
use std::collections::HashMap;

use crate::{
    compiler::compiler::Compiler,
    error::runtime_error::RuntimeError,
    runtime::object::Object,
    runtime::value::Value,
};

// ============================================================
//                         DICT()
// ============================================================

pub fn native_dict(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 0,
            found: args.len(),
        });
    }

    Ok(Value::new_dict(Vec::new()))
}

// ============================================================
//                         GET
// ============================================================

pub fn native_dict_get(
    args: &[Value],
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    args[0].dict_get(&args[1])
}

// ============================================================
//                         SET
// ============================================================

pub fn native_dict_set(
    args: &[Value],
) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 3,
            found: args.len(),
        });
    }

    args[0].dict_set(&args[1], args[2].clone())?;

    Ok(Value::Nil)
}

// ============================================================
//                         HAS
// ============================================================

pub fn native_dict_has(
    args: &[Value],
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    Ok(Value::Boolean(
        args[0].dict_contains(&args[1])?,
    ))
}

// ============================================================
//                        REMOVE
// ============================================================

pub fn native_dict_remove(
    args: &[Value],
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    args[0].dict_remove(&args[1])
}

// ============================================================
//                         LENGTH
// ============================================================

pub fn native_dict_length(
    args: &[Value],
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    Ok(Value::Integer(args[0].dict_len()? as i64))
}

// ============================================================
//                          KEYS
// ============================================================

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

// ============================================================
//                         VALUES
// ============================================================

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

// ============================================================
//                          ITEMS
// ============================================================

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

// ============================================================
//                          CLEAR
// ============================================================

pub fn native_dict_clear(
    args: &[Value],
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    args[0].dict_clear()?;

    Ok(Value::Nil)
}

// ============================================================
//                         GET_OR
// ============================================================

pub fn native_dict_get_or(
    args: &[Value],
) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 3,
            found: args.len(),
        });
    }

    if args[0].dict_contains(&args[1])? {
        args[0].dict_get(&args[1])
    } else {
        Ok(args[2].clone())
    }
}

// ============================================================
//                          UPDATE
// ============================================================

pub fn native_dict_update(
    args: &[Value],
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let entries = match &args[1] {
        Value::Object(handle) => {
            let object = handle.borrow();

            match &*object {
                Object::Dict(entries) => entries.clone(),
                _ => return Err(RuntimeError::TypeError),
            }
        }

        _ => return Err(RuntimeError::TypeError),
    };

    for (key, value) in entries {
        args[0].dict_set(&key, value)?;
    }

    Ok(Value::Nil)
}

// ============================================================
//                           COPY
// ============================================================

pub fn native_dict_copy(
    args: &[Value],
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let entries = match &args[0] {
        Value::Object(handle) => {
            let object = handle.borrow();

            match &*object {
                Object::Dict(entries) => entries.clone(),
                _ => return Err(RuntimeError::TypeError),
            }
        }

        _ => return Err(RuntimeError::TypeError),
    };

    Ok(Value::new_dict(entries))
}

// ============================================================
//                      METHOD DISPATCH
// ============================================================

pub fn dispatch_method(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, RuntimeError> {
    match name {
        "length" => Ok(Some(native_dict_length(args)?)),
        "get" => Ok(Some(native_dict_get(args)?)),
        "set" => Ok(Some(native_dict_set(args)?)),
        "has" => Ok(Some(native_dict_has(args)?)),
        "remove" => Ok(Some(native_dict_remove(args)?)),
        "keys" => Ok(Some(native_dict_keys(args)?)),
        "values" => Ok(Some(native_dict_values(args)?)),
        "items" => Ok(Some(native_dict_items(args)?)),
        "clear" => Ok(Some(native_dict_clear(args)?)),
        "get_or" => Ok(Some(native_dict_get_or(args)?)),
        "update" => Ok(Some(native_dict_update(args)?)),
        "copy" => Ok(Some(native_dict_copy(args)?)),
        _ => Ok(None),
    }
}

 
pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert(
        "dict".to_string(),
        Value::NativeFunction(native_dict),
    );
}

// ============================================================
//                  COMPILER NATIVE REGISTRATION
// ============================================================

pub fn register_compiler(compiler: &mut Compiler) {
    let _ = compiler.define_native("dict");
}
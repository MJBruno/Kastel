use std::collections::HashMap;

use crate::{
    compiler::compiler::Compiler, error::runtime_error::RuntimeError, runtime::object::Object,
    runtime::value::Value,
};

fn expect_index(value: &Value) -> Result<usize, RuntimeError> {
    match value {
        Value::Integer(index) if *index >= 0 => {
            usize::try_from(*index).map_err(|_| RuntimeError::TypeError)
        }

        Value::Float(index)
            if index.is_finite()
                && *index >= 0.0
                && index.fract() == 0.0
                && *index <= usize::MAX as f64 =>
        {
            Ok(*index as usize)
        }

        _ => Err(RuntimeError::ArrayIndexNotInteger),
    }
}

fn with_tuple<R>(
    value: &Value,
    callback: impl FnOnce(&Vec<Value>) -> Result<R, RuntimeError>,
) -> Result<R, RuntimeError> {
    match value {
        Value::Object(handle) => {
            let object = handle.borrow();

            match &*object {
                Object::Tuple(tuple) => callback(tuple),
                _ => Err(RuntimeError::TypeError),
            }
        }

        _ => Err(RuntimeError::TypeError),
    }
}

// ============================================================
//                     MÉTHODES (LECTURE SEULE)
// ============================================================
//
// Un tuple est immuable : contrairement à array.rs, il n'existe
// volontairement aucun native_push/pop/insert/remove/set/sort/clear.

pub fn native_is_empty(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    Ok(Value::Boolean(with_tuple(&args[0], |tuple| {
        Ok(tuple.is_empty())
    })?))
}

pub fn native_size(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    Ok(Value::Integer(with_tuple(&args[0], |tuple| {
        Ok(tuple.len() as i64)
    })?))
}

pub fn native_get(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let index = expect_index(&args[1])?;

    with_tuple(&args[0], |tuple| {
        tuple
            .get(index)
            .cloned()
            .ok_or(RuntimeError::ArrayIndexOutOfBounds {
                index,
                length: tuple.len(),
            })
    })
}

pub fn native_contains(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    with_tuple(&args[0], |tuple| {
        Ok(Value::Boolean(tuple.iter().any(|value| {
            Value::equals(value.clone(), args[1].clone())
        })))
    })
}

pub fn native_index_of(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    with_tuple(&args[0], |tuple| {
        let index = tuple
            .iter()
            .position(|value| Value::equals(value.clone(), args[1].clone()))
            .map(|index| index as i64)
            .unwrap_or(-1);

        Ok(Value::Integer(index))
    })
}

pub fn native_first(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    with_tuple(&args[0], |tuple| {
        Ok(tuple.first().cloned().unwrap_or(Value::None))
    })
}

pub fn native_last(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    with_tuple(&args[0], |tuple| {
        Ok(tuple.last().cloned().unwrap_or(Value::None))
    })
}

/// Convertit le tuple en un array (mutable) — la façon idiomatique
/// d'obtenir une copie modifiable d'un tuple.
pub fn native_to_list(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    with_tuple(&args[0], |tuple| Ok(Value::new_array(tuple.clone())))
}

// ============================================================
//                     METHOD DISPATCH
// ============================================================

pub fn dispatch_method(name: &str, args: &[Value]) -> Result<Option<Value>, RuntimeError> {
    let result = match name {
        // API standard des collections (un tuple est immuable : ni add,
        // ni remove, ni clear, ni copy).
        "size" => Some(native_size(args)?),
        "is_empty" => Some(native_is_empty(args)?),
        "to_string" => Some(super::to_string_method(args)?),

        // Nom supprimé.
        "length" => return Err(super::renamed_method_error("length", "size()")),

        "get" => Some(native_get(args)?),
        "contains" => Some(native_contains(args)?),
        "index_of" => Some(native_index_of(args)?),
        "first" => Some(native_first(args)?),
        "last" => Some(native_last(args)?),
        "to_list" => Some(native_to_list(args)?),

        // Nom supprimé : `Array` s'appelle désormais `List`.
        "to_array" => return Err(super::renamed_method_error("to_array", "to_list()")),

        _ => return Ok(None),
    };

    Ok(result)
}

pub fn register(globals: &mut HashMap<String, Value>) {
    let _ = globals;
}

pub fn register_compiler(compiler: &mut Compiler) {
    let _ = compiler;
}

use std::cmp::Ordering;
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

fn expect_slice_index(value: &Value, length: usize) -> Result<isize, RuntimeError> {
    let raw = match value {
        Value::Integer(value) => isize::try_from(*value).map_err(|_| RuntimeError::TypeError)?,

        Value::Float(value)
            if value.is_finite()
                && value.fract() == 0.0
                && *value >= isize::MIN as f64
                && *value <= isize::MAX as f64 =>
        {
            *value as isize
        }

        _ => return Err(RuntimeError::ArrayIndexNotInteger),
    };

    let len = length as isize;

    if raw < 0 {
        Ok((len + raw).max(0))
    } else {
        Ok(raw.min(len))
    }
}

fn with_array<R>(
    value: &Value,
    callback: impl FnOnce(&Vec<Value>) -> Result<R, RuntimeError>,
) -> Result<R, RuntimeError> {
    match value {
        Value::Object(handle) => {
            let object = handle.borrow();

            match &*object {
                Object::Array(array) => callback(array),
                _ => Err(RuntimeError::TypeError),
            }
        }

        _ => Err(RuntimeError::TypeError),
    }
}

fn with_array_mut<R>(
    value: &Value,
    callback: impl FnOnce(&mut Vec<Value>) -> Result<R, RuntimeError>,
) -> Result<R, RuntimeError> {
    match value {
        Value::Object(handle) => {
            let mut object = handle.borrow_mut();

            match &mut *object {
                Object::Array(array) => callback(array),
                _ => Err(RuntimeError::TypeError),
            }
        }

        _ => Err(RuntimeError::TypeError),
    }
}
#[allow(dead_code)]
pub fn native_dict_compat_array_placeholder(_args: &[Value]) -> Result<Value, RuntimeError> {
    Err(RuntimeError::NativeError)
}

// ============================================================
//                      LIST METHODS
// ============================================================
#[allow(dead_code)]
pub fn native_length(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    Ok(Value::Integer(with_array(&args[0], |array| {
        Ok(array.len() as i64)
    })?))
}

pub fn native_push(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let length = with_array_mut(&args[0], |array| {
        array.push(args[1].clone());
        Ok(array.len())
    })?;

    Ok(Value::Integer(length as i64))
}

pub fn native_pop(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    with_array_mut(&args[0], |array| Ok(array.pop().unwrap_or(Value::Nil)))
}

pub fn native_insert(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 3,
            found: args.len(),
        });
    }

    let index = expect_index(&args[1])?;

    let length = with_array_mut(&args[0], |array| {
        if index > array.len() {
            return Err(RuntimeError::ArrayIndexOutOfBounds {
                index,
                length: array.len(),
            });
        }

        array.insert(index, args[2].clone());

        Ok(array.len())
    })?;

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

    with_array_mut(&args[0], |array| {
        if index >= array.len() {
            return Err(RuntimeError::ArrayIndexOutOfBounds {
                index,
                length: array.len(),
            });
        }

        Ok(array.remove(index))
    })
}

pub fn native_get(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let index = expect_index(&args[1])?;

    with_array(&args[0], |array| {
        array
            .get(index)
            .cloned()
            .ok_or(RuntimeError::ArrayIndexOutOfBounds {
                index,
                length: array.len(),
            })
    })
}

pub fn native_set(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 3,
            found: args.len(),
        });
    }

    let index = expect_index(&args[1])?;

    with_array_mut(&args[0], |array| {
        if index >= array.len() {
            return Err(RuntimeError::ArrayIndexOutOfBounds {
                index,
                length: array.len(),
            });
        }

        array[index] = args[2].clone();

        Ok(Value::Nil)
    })
}

pub fn native_contains(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    with_array(&args[0], |array| {
        Ok(Value::Boolean(array.iter().any(|value| {
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

    with_array(&args[0], |array| {
        let index = array
            .iter()
            .position(|value| Value::equals(value.clone(), args[1].clone()))
            .map(|index| index as i64)
            .unwrap_or(-1);

        Ok(Value::Integer(index))
    })
}

pub fn native_slice(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 3,
            found: args.len(),
        });
    }

    with_array(&args[0], |array| {
        let start = expect_slice_index(&args[1], array.len())?;

        let end = expect_slice_index(&args[2], array.len())?;

        let start = start as usize;
        let end = end as usize;

        if start >= end {
            return Ok(Value::new_array(Vec::new()));
        }

        Ok(Value::new_array(array[start..end].to_vec()))
    })
}

pub fn native_reverse(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    with_array_mut(&args[0], |array| {
        array.reverse();
        Ok(args[0].clone())
    })
}

pub fn native_join(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let separator = args[1].as_string_value().ok_or(RuntimeError::TypeError)?;

    with_array(&args[0], |array| {
        let mut output = String::new();

        for (index, value) in array.iter().enumerate() {
            if index != 0 {
                output.push_str(&separator);
            }

            output.push_str(&value.to_string());
        }

        Ok(Value::new_string(output))
    })
}

pub fn native_clear(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    with_array_mut(&args[0], |array| {
        array.clear();
        Ok(Value::Nil)
    })
}

pub fn native_first(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    with_array(&args[0], |array| {
        Ok(array.first().cloned().unwrap_or(Value::Nil))
    })
}

pub fn native_last(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    with_array(&args[0], |array| {
        Ok(array.last().cloned().unwrap_or(Value::Nil))
    })
}

pub fn native_sort(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    with_array_mut(&args[0], |array| {
        array.sort_by(compare_values);
        Ok(args[0].clone())
    })
}

fn compare_values(left: &Value, right: &Value) -> Ordering {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => a.cmp(b),

        (Value::Integer(a), Value::Float(b)) => {
            (*a as f64).partial_cmp(b).unwrap_or(Ordering::Equal)
        }

        (Value::Float(a), Value::Integer(b)) => {
            a.partial_cmp(&(*b as f64)).unwrap_or(Ordering::Equal)
        }

        (Value::Float(a), Value::Float(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),

        (Value::Object(a), Value::Object(b)) => {
            let a_borrow = a.borrow();
            let b_borrow = b.borrow();

            match (&*a_borrow, &*b_borrow) {
                (Object::String(a), Object::String(b)) => a.cmp(b),

                _ => Ordering::Equal,
            }
        }

        (Value::Boolean(a), Value::Boolean(b)) => a.cmp(b),

        (Value::Nil, Value::Nil) => Ordering::Equal,

        _ => Ordering::Equal,
    }
}

pub fn native_copy(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    with_array(&args[0], |array| Ok(Value::new_array(array.clone())))
}

// ============================================================
//                     METHOD DISPATCH
// ============================================================

pub fn dispatch_method(name: &str, args: &[Value]) -> Result<Option<Value>, RuntimeError> {
    let result = match name {
        "push" => Some(native_push(args)?),
        "pop" => Some(native_pop(args)?),
        "insert" => Some(native_insert(args)?),
        "remove" => Some(native_remove(args)?),
        "get" => Some(native_get(args)?),
        "set" => Some(native_set(args)?),
        "contains" => Some(native_contains(args)?),
        "index_of" => Some(native_index_of(args)?),
        "slice" => Some(native_slice(args)?),
        "reverse" => Some(native_reverse(args)?),
        "join" => Some(native_join(args)?),
        "clear" => Some(native_clear(args)?),
        "first" => Some(native_first(args)?),
        "last" => Some(native_last(args)?),
        "sort" => Some(native_sort(args)?),
        "copy" => Some(native_copy(args)?),

        // Ces méthodes sont exécutées directement par la VM.
        "map" | "filter" | "reduce" | "any" | "all" => None,

        _ => return Ok(None),
    };

    Ok(result)
}

// Les anciennes natives globales ne sont plus exposées.
// Les listes sont manipulées par leur API objet.

pub fn register(globals: &mut HashMap<String, Value>) {
    let _ = globals;
}

pub fn register_compiler(compiler: &mut Compiler) {
    let _ = compiler;
}

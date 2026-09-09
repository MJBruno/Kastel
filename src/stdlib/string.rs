use std::collections::HashMap;

use crate::{
    compiler::compiler::Compiler,
    error::runtime_error::RuntimeError,
    runtime::value::Value,
};

// ============================================================
//                         HELPERS
// ============================================================

fn expect_string(value: &Value) -> Result<String, RuntimeError> {
    value
        .as_string_value()
        .ok_or(RuntimeError::TypeError)
}

fn expect_integer(value: &Value) -> Result<i64, RuntimeError> {
    match value {
        Value::Integer(value) => Ok(*value),
        _ => Err(RuntimeError::TypeError),
    }
}

fn expect_index(value: &Value) -> Result<usize, RuntimeError> {
    let index = expect_integer(value)?;

    if index < 0 {
        return Err(RuntimeError::IndexOutOfBounds);
    }

    usize::try_from(index).map_err(|_| RuntimeError::IndexOutOfBounds)
}

fn expect_non_negative_count(value: &Value) -> Result<usize, RuntimeError> {
    let count = expect_integer(value)?;

    if count < 0 {
        return Err(RuntimeError::TypeError);
    }

    usize::try_from(count).map_err(|_| RuntimeError::TypeError)
}

// ============================================================
//                         FORMAT
// ============================================================

pub(crate) fn format_string(
    format: &str,
    args: &[Value],
) -> Result<String, RuntimeError> {
    let mut result = String::with_capacity(format.len());

    let mut chars = format.chars().peekable();
    let mut arg_index = 0;

    while let Some(character) = chars.next() {
        if character == '{' && chars.peek() == Some(&'}') {
            chars.next();

            let value = args.get(arg_index).ok_or(
                RuntimeError::WrongArgumentCount {
                    expected: arg_index + 1,
                    found: args.len(),
                },
            )?;

            result.push_str(&value.to_string());
            arg_index += 1;
        } else {
            result.push(character);
        }
    }

    if arg_index < args.len() {
        return Err(RuntimeError::WrongArgumentCount {
            expected: arg_index,
            found: args.len(),
        });
    }

    Ok(result)
}

// ============================================================
//                         GLOBAL NATIVES
// ============================================================

pub fn native_format(args: &[Value]) -> Result<Value, RuntimeError> {
    let Some(first) = args.first() else {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: 0,
        });
    };

    let format = expect_string(first)?;

    Ok(Value::new_string(
        format_string(&format, &args[1..])?,
    ))
}

// pub fn native_strlen(args: &[Value]) -> Result<Value, RuntimeError> {
//     if args.len() != 1 {
//         return Err(RuntimeError::WrongArgumentCount {
//             expected: 1,
//             found: args.len(),
//         });
//     }

//     let value = expect_string(&args[0])?;

//     Ok(Value::Integer(value.chars().count() as i64))
// }

pub fn native_lower(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    Ok(Value::new_string(
        expect_string(&args[0])?.to_lowercase(),
    ))
}

pub fn native_upper(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    Ok(Value::new_string(
        expect_string(&args[0])?.to_uppercase(),
    ))
}

pub fn native_trim(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    Ok(Value::new_string(
        expect_string(&args[0])?.trim().to_string(),
    ))
}

pub fn native_starts_with(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;
    let prefix = expect_string(&args[1])?;

    Ok(Value::Boolean(value.starts_with(&prefix)))
}

pub fn native_ends_with(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;
    let suffix = expect_string(&args[1])?;

    Ok(Value::Boolean(value.ends_with(&suffix)))
}

pub fn native_contains(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;
    let search = expect_string(&args[1])?;

    Ok(Value::Boolean(value.contains(&search)))
}

pub fn native_replace(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 3,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;
    let from = expect_string(&args[1])?;
    let to = expect_string(&args[2])?;

    Ok(Value::new_string(value.replacen(&from, &to, 1)))
}

pub fn native_split(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;
    let separator = expect_string(&args[1])?;

    let elements = value
        .split(&separator)
        .map(|part| Value::new_string(part.to_string()))
        .collect();

    Ok(Value::new_array(elements))
}

// ============================================================
//                       STRING METHODS
// ============================================================

pub fn native_length(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;

    Ok(Value::Integer(value.chars().count() as i64))
}

pub fn native_get(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;
    let index = expect_index(&args[1])?;

    value
        .chars()
        .nth(index)
        .map(|character| Value::new_string(character.to_string()))
        .ok_or(RuntimeError::IndexOutOfBounds)
}

pub fn native_contains_method(args: &[Value]) -> Result<Value, RuntimeError> {
    native_contains(args)
}

pub fn native_starts_with_method(args: &[Value]) -> Result<Value, RuntimeError> {
    native_starts_with(args)
}

pub fn native_ends_with_method(args: &[Value]) -> Result<Value, RuntimeError> {
    native_ends_with(args)
}

pub fn native_index_of(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;
    let search = expect_string(&args[1])?;

    let index = value
        .find(&search)
        .map(|byte_index| value[..byte_index].chars().count() as i64)
        .unwrap_or(-1);

    Ok(Value::Integer(index))
}

pub fn native_last_index_of(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;
    let search = expect_string(&args[1])?;

    let index = value
        .rfind(&search)
        .map(|byte_index| value[..byte_index].chars().count() as i64)
        .unwrap_or(-1);

    Ok(Value::Integer(index))
}

pub fn native_slice(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 3,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;
    let start = expect_integer(&args[1])?;
    let end = expect_integer(&args[2])?;

    let length = value.chars().count() as i64;

    let normalize = |index: i64| -> usize {
        if index < 0 {
            (length + index).max(0) as usize
        } else {
            index.min(length) as usize
        }
    };

    let start = normalize(start);
    let end = normalize(end);

    if start >= end {
        return Ok(Value::new_string(String::new()));
    }

    let result: String = value
        .chars()
        .skip(start)
        .take(end - start)
        .collect();

    Ok(Value::new_string(result))
}

pub fn native_substring(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 3,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;
    let start = expect_integer(&args[1])?;
    let length = expect_integer(&args[2])?;

    if start < 0 || length < 0 {
        return Err(RuntimeError::TypeError);
    }

    let start = start as usize;
    let length = length as usize;

    let result: String = value
        .chars()
        .skip(start)
        .take(length)
        .collect();

    Ok(Value::new_string(result))
}

pub fn native_trim_start(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    Ok(Value::new_string(
        expect_string(&args[0])?.trim_start().to_string(),
    ))
}

pub fn native_trim_end(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    Ok(Value::new_string(
        expect_string(&args[0])?.trim_end().to_string(),
    ))
}

pub fn native_replace_all(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 3,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;
    let from = expect_string(&args[1])?;
    let to = expect_string(&args[2])?;

    Ok(Value::new_string(value.replace(&from, &to)))
}

pub fn native_join(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let separator = expect_string(&args[0])?;

    let values = match &args[1] {
        Value::Object(handle) => {
            let object = handle.borrow();

            match &*object {
                crate::runtime::object::Object::Array(values) => {
                    values.clone()
                }

                _ => return Err(RuntimeError::TypeError),
            }
        }

        _ => return Err(RuntimeError::TypeError),
    };

    let mut result = String::new();

    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            result.push_str(&separator);
        }

        result.push_str(&value.to_string());
    }

    Ok(Value::new_string(result))
}

pub fn native_repeat(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;
    let count = expect_non_negative_count(&args[1])?;

    Ok(Value::new_string(value.repeat(count)))
}

pub fn native_char_at(args: &[Value]) -> Result<Value, RuntimeError> {
    native_get(args)
}

pub fn native_to_int(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;

    let parsed = value
        .trim()
        .parse::<i64>()
        .map_err(|_| RuntimeError::TypeError)?;

    Ok(Value::Integer(parsed))
}

pub fn native_to_float(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;

    let parsed = value
        .trim()
        .parse::<f64>()
        .map_err(|_| RuntimeError::TypeError)?;

    Ok(Value::Float(parsed))
}

pub fn native_is_empty(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;

    Ok(Value::Boolean(value.is_empty()))
}

pub fn native_is_digit(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;

    Ok(Value::Boolean(
        !value.is_empty() && value.chars().all(|c| c.is_ascii_digit()),
    ))
}

pub fn native_is_alpha(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;

    Ok(Value::Boolean(
        !value.is_empty() && value.chars().all(|c| c.is_alphabetic()),
    ))
}

pub fn native_is_alphanumeric(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;

    Ok(Value::Boolean(
        !value.is_empty() && value.chars().all(|c| c.is_alphanumeric()),
    ))
}

pub fn native_reverse(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;

    let result: String = value.chars().rev().collect();

    Ok(Value::new_string(result))
}

// ============================================================
//                     METHOD DISPATCH
// ============================================================

pub fn dispatch_method(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, RuntimeError> {
    let result = match name {
        "length" => Some(native_length(args)?),

        "get" => Some(native_get(args)?),

        "contains" => Some(native_contains_method(args)?),

        "starts_with" => Some(native_starts_with_method(args)?),

        "ends_with" => Some(native_ends_with_method(args)?),

        "index_of" => Some(native_index_of(args)?),

        "last_index_of" => Some(native_last_index_of(args)?),

        "slice" => Some(native_slice(args)?),

        "substring" => Some(native_substring(args)?),

        "upper" => Some(native_upper(args)?),

        "lower" => Some(native_lower(args)?),

        "trim" => Some(native_trim(args)?),

        "trim_start" => Some(native_trim_start(args)?),

        "trim_end" => Some(native_trim_end(args)?),

        "replace" => Some(native_replace(args)?),

        "replace_all" => Some(native_replace_all(args)?),

        "split" => Some(native_split(args)?),

        "join" => Some(native_join(args)?),

        "repeat" => Some(native_repeat(args)?),

        "char_at" => Some(native_char_at(args)?),

        "to_int" => Some(native_to_int(args)?),

        "to_float" => Some(native_to_float(args)?),

        "is_empty" => Some(native_is_empty(args)?),

        "is_digit" => Some(native_is_digit(args)?),

        "is_alpha" => Some(native_is_alpha(args)?),

        "is_alphanumeric" => Some(native_is_alphanumeric(args)?),

        "reverse" => Some(native_reverse(args)?),

        _ => return Ok(None),
    };

    Ok(result)
}

// ============================================================
//                      GLOBAL REGISTRATION
// ============================================================

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert(
        "format".to_string(),
        Value::NativeFunction(native_format),
    );
}

pub fn register_compiler(compiler: &mut Compiler) {
    let _ = compiler.define_native("format");
}
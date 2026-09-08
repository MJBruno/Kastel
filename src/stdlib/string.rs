use std::collections::HashMap;

use crate::{
    compiler::compiler::Compiler,
    error::runtime_error::RuntimeError,
    runtime::value::Value,
};

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

            let value = args
                .get(arg_index)
                .ok_or(RuntimeError::WrongArgumentCount {
                    expected: arg_index + 1,
                    found: args.len(),
                })?;

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

fn expect_string(value: &Value) -> Result<String, RuntimeError> {
    value
        .as_string_value()
        .ok_or(RuntimeError::TypeError)
}

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

pub fn native_strlen(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;

    Ok(Value::Integer(value.chars().count() as i64))
}

pub fn native_lower(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    Ok(Value::new_string(expect_string(&args[0])?.to_lowercase()))
}

pub fn native_upper(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    Ok(Value::new_string(expect_string(&args[0])?.to_uppercase()))
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

    Ok(Value::new_string(value.replace(&from, &to)))
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

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert(
        "format".to_string(),
        Value::NativeFunction(native_format),
    );

    globals.insert(
        "strlen".to_string(),
        Value::NativeFunction(native_strlen),
    );

    globals.insert(
        "lower".to_string(),
        Value::NativeFunction(native_lower),
    );

    globals.insert(
        "upper".to_string(),
        Value::NativeFunction(native_upper),
    );

    globals.insert(
        "trim".to_string(),
        Value::NativeFunction(native_trim),
    );

    globals.insert(
        "starts_with".to_string(),
        Value::NativeFunction(native_starts_with),
    );

    globals.insert(
        "ends_with".to_string(),
        Value::NativeFunction(native_ends_with),
    );

    globals.insert(
        "includes".to_string(),
        Value::NativeFunction(native_contains),
    );

    globals.insert(
        "replace".to_string(),
        Value::NativeFunction(native_replace),
    );

    globals.insert(
        "split".to_string(),
        Value::NativeFunction(native_split),
    );
}

pub fn register_compiler(compiler: &mut Compiler) {
    let names = [
        "format",
        "strlen",
        "lower",
        "upper",
        "trim",
        "starts_with",
        "ends_with",
        "includes",
        "replace",
        "split",
    ];

    for name in names {
        let _ = compiler.define_native(name);
    }
}


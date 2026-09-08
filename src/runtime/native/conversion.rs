use crate::{
    error::runtime_error::RuntimeError,
    runtime::object::Object,
    runtime::value::Value,
};

fn parse_number_like(value: &Value) -> Result<f64, RuntimeError> {
    match value {
        Value::Integer(n) => Ok(*n as f64),

        Value::Float(n) => Ok(*n),

        Value::Boolean(b) => Ok(if *b { 1.0 } else { 0.0 }),

        _ => {
            if let Some(s) = value.as_string_value() {
                s.trim()
                    .parse::<f64>()
                    .map_err(|_| RuntimeError::TypeError)
            } else {
                Err(RuntimeError::TypeError)
            }
        }
    }
}

pub fn native_int(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    if let Value::Integer(n) = &args[0] {
        return Ok(Value::Integer(*n));
    }

    let value = parse_number_like(&args[0])?;

    if !value.is_finite()
        || value < i64::MIN as f64
        || value > i64::MAX as f64
    {
        return Err(RuntimeError::TypeError);
    }

    Ok(Value::Integer(value.trunc() as i64))
}

pub fn native_float(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let value = parse_number_like(&args[0])?;

    Ok(Value::Float(value))
}

pub fn native_str(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    Ok(Value::new_string(args[0].to_string()))
}

pub fn native_bool(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    Ok(Value::Boolean(args[0].is_truthy()))
}

pub fn native_type(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let name = match &args[0] {
        Value::Integer(_) => "int",
        Value::Float(_) => "float",
        Value::Boolean(_) => "bool",
        Value::Nil => "nil",
        Value::Range { .. } => "range",
        Value::NativeFunction(_) => "function",

        Value::Object(handle) => match &*handle.borrow() {
            Object::String(_) => "string",
            Object::Array(_) => "array",
            Object::Dict(_) => "object",
            Object::Function(_) | Object::Closure(_) => "function",
            Object::Iterator(_) => "iterator",
            Object::Module(_) => "module",
        },
    };

    Ok(Value::new_string(name.to_string()))
}
use std::collections::HashMap;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    compiler::compiler::Compiler, error::runtime_error::RuntimeError, runtime::object::Object,
    runtime::value::Value,
};

fn parse_number(value: &Value) -> Result<f64, RuntimeError> {
    match value {
        Value::Integer(value) => Ok(*value as f64),

        Value::Float(value) => Ok(*value),

        Value::Boolean(value) => Ok(if *value { 1.0 } else { 0.0 }),

        _ => {
            let string = value.as_string_value().ok_or(RuntimeError::TypeError)?;

            string
                .trim()
                .parse::<f64>()
                .map_err(|_| RuntimeError::TypeError)
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

    if let Value::Integer(value) = args[0] {
        return Ok(Value::Integer(value));
    }

    let value = parse_number(&args[0])?;

    // i64 est représenté en f64 avec une borne supérieure
    // exclusive de 2^63.
    const I64_MAX_EXCLUSIVE: f64 = 9_223_372_036_854_775_808.0;

    if !value.is_finite() || value < i64::MIN as f64 || value >= I64_MAX_EXCLUSIVE {
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

    Ok(Value::Float(parse_number(&args[0])?))
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

    Ok(Value::new_string(name.to_owned()))
}

pub fn native_clock(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 0,
            found: args.len(),
        });
    }

    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| RuntimeError::NativeError)?;

    Ok(Value::Float(elapsed.as_secs_f64()))
}

pub fn native_cwd(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 0,
            found: args.len(),
        });
    }

    let path = env::current_dir().map_err(|_| RuntimeError::NativeError)?;

    Ok(Value::new_string(path.to_string_lossy().into_owned()))
}

pub fn native_env(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let name = args[0].as_string_value().ok_or(RuntimeError::TypeError)?;

    match env::var(name) {
        Ok(value) => Ok(Value::new_string(value)),

        Err(env::VarError::NotPresent) => Ok(Value::Nil),

        Err(env::VarError::NotUnicode(_)) => Err(RuntimeError::NativeError),
    }
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert("int".to_string(), Value::NativeFunction(native_int));

    globals.insert("float".to_string(), Value::NativeFunction(native_float));

    globals.insert("str".to_string(), Value::NativeFunction(native_str));

    globals.insert("bool".to_string(), Value::NativeFunction(native_bool));

    globals.insert("type".to_string(), Value::NativeFunction(native_type));

    globals.insert("clock".to_string(), Value::NativeFunction(native_clock));

    globals.insert("cwd".to_string(), Value::NativeFunction(native_cwd));

    globals.insert("env".to_string(), Value::NativeFunction(native_env));
}

pub fn register_compiler(compiler: &mut Compiler) {
    let names = ["int", "float", "str", "bool", "type", "clock", "cwd", "env"];

    for name in names {
        let _ = compiler.define_native(name);
    }
}

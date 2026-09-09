use std::collections::HashMap;

use crate::{
    compiler::compiler::Compiler,
    error::runtime_error::RuntimeError,
    runtime::value::Value,
};

pub fn native_dict(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 0,
            found: args.len(),
        });
    }

    Ok(Value::new_dict(Vec::new()))
}

pub fn native_dict_get(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    args[0].dict_get(&args[1])
}

pub fn native_dict_set(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 3,
            found: args.len(),
        });
    }

    args[0].dict_set(&args[1], args[2].clone())?;

    Ok(Value::Nil)
}

pub fn native_dict_has(args: &[Value]) -> Result<Value, RuntimeError> {
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

pub fn native_dict_remove(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    args[0].dict_remove(&args[1])
}

pub fn native_dict_keys(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    args[0].dict_keys()
}

pub fn native_dict_values(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    args[0].dict_values()
}

pub fn native_dict_items(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    args[0].dict_items()
}

pub fn native_dict_clear(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    args[0].dict_clear()?;

    Ok(Value::Nil)
}

/// Seul `dict()` est exposé comme native globale.
/// Les méthodes sont compilées comme appels de méthode.
pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert(
        "dict".to_string(),
        Value::NativeFunction(native_dict),
    );
}

pub fn register_compiler(compiler: &mut Compiler) {
    let _ = compiler.define_native("dict");
}
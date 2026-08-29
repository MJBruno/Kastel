use crate::{compile::compiler::Compiler, error::runtime_error::RuntimeError, runtime::value::Value};
 

use std::time::{SystemTime, UNIX_EPOCH};

pub type NativeFn = fn(&[Value]) -> Result<Value, RuntimeError>;

// ================================================================
// CLOCK
// ================================================================

pub fn native_clock(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 0,
            found: args.len(),
        });
    }

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| RuntimeError::TypeError)?;

    Ok(Value::Number(duration.as_secs_f64()))
}

// ================================================================
// ADD
// ================================================================

pub fn native_add(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let a = match &args[0] {
        Value::Number(value) => *value,

        _ => {
            return Err(RuntimeError::TypeError);
        }
    };

    let b = match &args[1] {
        Value::Number(value) => *value,

        _ => {
            return Err(RuntimeError::TypeError);
        }
    };

    Ok(Value::Number(a + b))
}

// ================================================================
// PRINT
// ================================================================

pub fn native_print(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    println!("{}", args[0]);

    Ok(args[0].clone())
}

// ================================================================
// ARRAY PUSH
//
// push(array, value)
//
// Retourne la nouvelle longueur.
// ================================================================

pub fn native_array_push(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let array = &args[0];
    let value = args[1].clone();

    let length = array.array_push(value)?;

    Ok(Value::Number(length as f64))
}

// ================================================================
// ARRAY POP
//
// pop(array)
//
// Retourne le dernier élément.
// Tableau vide => nil.
// ================================================================

pub fn native_array_pop(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    args[0].array_pop()
}

// ================================================================
// ARRAY LENGTH
//
// length(array)
// ================================================================

pub fn native_array_length(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let length = args[0].array_len()?;

    Ok(Value::Number(length as f64))
}

pub fn native_array_insert(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 3,
            found: args.len(),
        });
    }

    let index = match args[1] {
        Value::Number(value) => {
            if value < 0.0 || value.fract() != 0.0 {
                return Err(RuntimeError::TypeError);
            }

            value as usize
        }

        _ => {
            return Err(RuntimeError::TypeError);
        }
    };

    let length = args[0].array_insert(index, args[2].clone())?;

    Ok(Value::Number(length as f64))
}

pub fn native_array_remove(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let index = match args[1] {
        Value::Number(value) => {
            if value < 0.0 || value.fract() != 0.0 {
                return Err(RuntimeError::TypeError);
            }

            value as usize
        }

        _ => {
            return Err(RuntimeError::TypeError);
        }
    };

    args[0].array_remove(index)
}

// ================================================================
// REGISTER NATIVES
// ================================================================

pub fn register_natives(globals: &mut std::collections::HashMap<String, Value>) {
    globals.insert("clock".to_string(), Value::NativeFunction(native_clock));

    globals.insert("native_add".to_string(), Value::NativeFunction(native_add));

    globals.insert("println".to_string(), Value::NativeFunction(native_print));

    globals.insert("push".to_string(), Value::NativeFunction(native_array_push));

    globals.insert("pop".to_string(), Value::NativeFunction(native_array_pop));

    globals.insert(
        "length".to_string(),
        Value::NativeFunction(native_array_length),
    );
    globals.insert(
        "insert".to_string(),
        Value::NativeFunction(native_array_insert),
    );

    globals.insert(
        "remove".to_string(),
        Value::NativeFunction(native_array_remove),
    );
}

// ================================================================
// COMPILER REGISTRATION
// ================================================================

pub fn execute_native(compiler: &mut Compiler) {
    let _ = compiler.define_native("clock");

    let _ = compiler.define_native("native_add");

    let _ = compiler.define_native("println");

    let _ = compiler.define_native("push");

    let _ = compiler.define_native("pop");

    let _ = compiler.define_native("length");

    let _ = compiler.define_native("insert");
    
    let _ = compiler.define_native("remove");
}

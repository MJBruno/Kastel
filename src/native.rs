use crate::value::Value;
use crate::{compiler::Compiler, error::RuntimeError};

use std::time::{SystemTime, UNIX_EPOCH};

pub type NativeFn = fn(&[Value]) -> Result<Value, RuntimeError>;

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

    Ok(Value::Number(duration.as_secs() as f64))
}

pub fn native_add(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let a = match &args[0] {
        Value::Number(value) => *value,
        _ => return Err(RuntimeError::TypeError),
    };

    let b = match &args[1] {
        Value::Number(value) => *value,
        _ => return Err(RuntimeError::TypeError),
    };

    Ok(Value::Number(a + b))
}

#[allow(dead_code)]
pub fn native_print(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    match &args[0] {
        Value::Number(n) => println!("{}", n),
        Value::Boolean(b) => println!("{}", b),
        Value::String(s) => println!("{}", s),
        Value::Nil => println!("nil"),
        Value::Function(function) => println!("<func: {}>", function.name),
        Value::Closure(closure) => {
            let closure = closure.borrow();
            println!("<closure '{}'>", closure.function.name)
        }
        Value::NativeFunction(function) => {
            println!("<fun '{:?} '>", function)
        }
        #[allow(unused_variables)]
        Value::Array(ref_cell) => todo!(),
    };

    Ok(args[0].clone())
}

pub fn register_natives(globals: &mut std::collections::HashMap<String, Value>) {
    globals.insert("clock".to_string(), Value::NativeFunction(native_clock));
    globals.insert("native_add".to_string(), Value::NativeFunction(native_add));
    globals.insert("println".to_string(), Value::NativeFunction(native_print));
}

pub fn execute_native(compiler: &mut Compiler) {
    let _ = compiler.define_native("clock");
    let _ = compiler.define_native("native_add");
    let _ = compiler.define_native("println");
}

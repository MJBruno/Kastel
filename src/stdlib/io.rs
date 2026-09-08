
use std::collections::HashMap;
use std::io::{self, Write};

use crate::{
    compiler::compiler::Compiler,
    error::runtime_error::RuntimeError,
    runtime::value::Value,
};

use super::string::format_string;

fn render_arguments(args: &[Value]) -> Result<String, RuntimeError> {
    let Some(first) = args.first() else {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: 0,
        });
    };

    if args.len() == 1 {
        if let Some(text) = first.as_string_value() {
            return Ok(text);
        }

        return Ok(first.to_string());
    }

    let format = first
        .as_string_value()
        .ok_or(RuntimeError::TypeError)?;

    format_string(&format, &args[1..])
}

pub fn native_println(args: &[Value]) -> Result<Value, RuntimeError> {
    let formatted = render_arguments(args)?;

    println!("{formatted}");

    Ok(Value::new_string(formatted))
}

pub fn native_print(args: &[Value]) -> Result<Value, RuntimeError> {
    let formatted = render_arguments(args)?;

    print!("{formatted}");

    io::stdout()
        .flush()
        .map_err(|_| RuntimeError::NativeError)?;

    Ok(Value::new_string(formatted))
}

pub fn native_input(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() > 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    if let Some(prompt) = args.first() {
        print!("{prompt}");

        io::stdout()
            .flush()
            .map_err(|_| RuntimeError::NativeError)?;
    }

    let mut buffer = String::new();

    io::stdin()
        .read_line(&mut buffer)
        .map_err(|_| RuntimeError::NativeError)?;

    let value = buffer
        .trim_end_matches(['\n', '\r'])
        .to_string();

    Ok(Value::new_string(value))
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert(
        "print".to_string(),
        Value::NativeFunction(native_print),
    );

    globals.insert(
        "println".to_string(),
        Value::NativeFunction(native_println),
    );

    globals.insert(
        "input".to_string(),
        Value::NativeFunction(native_input),
    );
}

pub fn register_compiler(compiler: &mut Compiler) {
    let _ = compiler.define_native("print");
    let _ = compiler.define_native("println");
    let _ = compiler.define_native("input");
}


use std::io::{self, Write};

use crate::{
    error::runtime_error::RuntimeError,
    runtime::value::Value,
};

fn format_string(
    format: &str,
    args: &[Value],
) -> Result<String, RuntimeError> {
    let mut result = String::with_capacity(format.len());
    let mut chars = format.chars().peekable();
    let mut arg_index = 0;

    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'}') {
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
            result.push(c);
        }
    }

    Ok(result)
}

pub fn native_println(
    args: &[Value],
) -> Result<Value, RuntimeError> {
    let formatted = render_arguments(args)?;

    println!("{formatted}");

    Ok(Value::new_string(formatted))
}

pub fn native_print(
    args: &[Value],
) -> Result<Value, RuntimeError> {
    let formatted = render_arguments(args)?;

    print!("{formatted}");

    io::stdout().flush().ok();

    Ok(Value::new_string(formatted))
}

fn render_arguments(
    args: &[Value],
) -> Result<String, RuntimeError> {
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

pub fn native_input(
    args: &[Value],
) -> Result<Value, RuntimeError> {
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

    let trimmed =
        buffer.trim_end_matches(['\n', '\r']).to_string();

    Ok(Value::new_string(trimmed))
}

pub fn native_format(
    args: &[Value],
) -> Result<Value, RuntimeError> {
    let Some(first) = args.first() else {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: 0,
        });
    };

    let format = first
        .as_string_value()
        .ok_or(RuntimeError::TypeError)?;

    let formatted =
        format_string(&format, &args[1..])?;

    Ok(Value::new_string(formatted))
}
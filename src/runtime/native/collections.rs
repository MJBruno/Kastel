use crate::{
    error::runtime_error::RuntimeError,
    runtime::value::Value,
};

pub fn native_list(
    args: &[Value],
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    crate::runtime::iterator::drain_to_array(&args[0])
}

pub fn native_array_push(
    args: &[Value],
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let array = &args[0];
    let value = args[1].clone();

    let length = array.array_push(value)?;

    Ok(Value::Integer(length as i64))
}

pub fn native_array_pop(
    args: &[Value],
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    args[0].array_pop()
}

pub fn native_array_length(
    args: &[Value],
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let length = args[0].array_len()?;

    Ok(Value::Integer(length as i64))
}

pub fn native_array_insert(
    args: &[Value],
) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 3,
            found: args.len(),
        });
    }

    let index = expect_array_index(&args[1])?;

    let length = args[0].array_insert(
        index,
        args[2].clone(),
    )?;

    Ok(Value::Integer(length as i64))
}

pub fn native_array_remove(
    args: &[Value],
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let index = expect_array_index(&args[1])?;

    args[0].array_remove(index)
}

fn expect_array_index(
    value: &Value,
) -> Result<usize, RuntimeError> {
    match value {
        Value::Integer(n) if *n >= 0 => {
            Ok(*n as usize)
        }

        Value::Float(n)
            if n.is_finite()
                && *n >= 0.0
                && n.fract() == 0.0
                && *n <= usize::MAX as f64 =>
        {
            Ok(*n as usize)
        }

        _ => Err(RuntimeError::TypeError),
    }
}
//! Méthodes de `range(...)` : `size()`, `is_empty()`, `start()`, `stop()`,
//! `step()`, `to_string()`. Les autres méthodes (`map`, `filter`, `take`…)
//! passent par l'itérateur (voir `VirtualMachine::invoke_iterator_method`).

use crate::{
    error::runtime_error::RuntimeError,
    runtime::value::Value,
    stdlib::spec::{MethodTable, ParamTag, ReturnTag, method, vm_method},
};

/// `(start, stop, step)` du receveur, ou `TypeError` s'il n'est pas un range.
fn bounds(args: &[Value], expected: usize) -> Result<(i128, i128, i128), RuntimeError> {
    if args.len() != expected {
        return Err(RuntimeError::WrongArgumentCount {
            expected: expected.saturating_sub(1),
            found: args.len().saturating_sub(1),
        });
    }

    match args.first() {
        // `range()` n'accepte que des entiers et un pas non nul.
        Some(Value::Range { start, stop, step }) => {
            Ok((*start as i128, *stop as i128, *step as i128))
        }

        _ => Err(RuntimeError::TypeError),
    }
}

fn length(start: i128, stop: i128, step: i128) -> i128 {
    if step > 0 && start < stop {
        (stop - start + step - 1) / step
    } else if step < 0 && start > stop {
        (start - stop + (-step) - 1) / (-step)
    } else {
        0
    }
}

pub fn native_size(args: &[Value]) -> Result<Value, RuntimeError> {
    let (start, stop, step) = bounds(args, 1)?;

    Ok(Value::Integer(length(start, stop, step) as i64))
}

pub fn native_is_empty(args: &[Value]) -> Result<Value, RuntimeError> {
    let (start, stop, step) = bounds(args, 1)?;

    Ok(Value::Boolean(length(start, stop, step) == 0))
}

pub fn native_start(args: &[Value]) -> Result<Value, RuntimeError> {
    let (start, _, _) = bounds(args, 1)?;

    Ok(Value::Integer(start as i64))
}

pub fn native_stop(args: &[Value]) -> Result<Value, RuntimeError> {
    let (_, stop, _) = bounds(args, 1)?;

    Ok(Value::Integer(stop as i64))
}

pub fn native_step(args: &[Value]) -> Result<Value, RuntimeError> {
    let (_, _, step) = bounds(args, 1)?;

    Ok(Value::Integer(step as i64))
}

pub const METHODS: MethodTable = MethodTable {
    methods: &[
        method("size", native_size, &[], ReturnTag::Int),
        method("is_empty", native_is_empty, &[], ReturnTag::Bool),
        method("start", native_start, &[], ReturnTag::Int),
        method("stop", native_stop, &[], ReturnTag::Int),
        method("step", native_step, &[], ReturnTag::Int),
        method("to_string", super::to_string_method, &[], ReturnTag::Str),
        vm_method("iter", &[], ReturnTag::Dynamic),
    ],
    renamed: &[],
    strict: false,
};

pub fn dispatch_method(name: &str, args: &[Value]) -> Result<Option<Value>, RuntimeError> {
    METHODS.dispatch(name, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn range(start: f64, stop: f64, step: f64) -> Value {
        Value::Range { start, stop, step }
    }

    fn size(value: Value) -> i64 {
        match native_size(&[value]).unwrap() {
            Value::Integer(size) => size,
            other => panic!("entier attendu, reçu {other:?}"),
        }
    }

    #[test]
    fn size_counts_the_produced_values() {
        assert_eq!(size(range(0.0, 10.0, 2.0)), 5);
        assert_eq!(size(range(0.0, 10.0, 3.0)), 4);
        assert_eq!(size(range(10.0, 0.0, -2.0)), 5);
        assert_eq!(size(range(5.0, 5.0, 1.0)), 0);
        assert_eq!(size(range(5.0, 0.0, 1.0)), 0);
    }

    #[test]
    fn a_non_range_receiver_is_a_type_error() {
        assert!(matches!(
            native_size(&[Value::Integer(3)]),
            Err(RuntimeError::TypeError)
        ));
    }
}

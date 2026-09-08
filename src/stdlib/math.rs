use std::collections::HashMap;

use crate::{
    compiler::compiler::Compiler,
    error::runtime_error::RuntimeError,
    runtime::value::Value,
};

fn expect_number(value: &Value) -> Result<f64, RuntimeError> {
    match value {
        Value::Integer(value) => Ok(*value as f64),
        Value::Float(value) => Ok(*value),
        _ => Err(RuntimeError::TypeError),
    }
}

fn unary_number(
    args: &[Value],
    operation: fn(f64) -> f64,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let value = expect_number(&args[0])?;

    Ok(Value::Float(operation(value)))
}

pub fn native_abs(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    match args[0] {
        Value::Integer(value) => value
            .checked_abs()
            .map(Value::Integer)
            .ok_or(RuntimeError::TypeError),

        Value::Float(value) => Ok(Value::Float(value.abs())),

        _ => Err(RuntimeError::TypeError),
    }
}

pub fn native_floor(args: &[Value]) -> Result<Value, RuntimeError> {
    unary_number(args, f64::floor)
}

pub fn native_ceil(args: &[Value]) -> Result<Value, RuntimeError> {
    unary_number(args, f64::ceil)
}

pub fn native_round(args: &[Value]) -> Result<Value, RuntimeError> {
    unary_number(args, f64::round)
}

pub fn native_sqrt(args: &[Value]) -> Result<Value, RuntimeError> {
    unary_number(args, f64::sqrt)
}

pub fn native_exp(args: &[Value]) -> Result<Value, RuntimeError> {
    unary_number(args, f64::exp)
}

pub fn native_log(args: &[Value]) -> Result<Value, RuntimeError> {
    unary_number(args, f64::ln)
}

pub fn native_log10(args: &[Value]) -> Result<Value, RuntimeError> {
    unary_number(args, f64::log10)
}

pub fn native_sin(args: &[Value]) -> Result<Value, RuntimeError> {
    unary_number(args, f64::sin)
}

pub fn native_cos(args: &[Value]) -> Result<Value, RuntimeError> {
    unary_number(args, f64::cos)
}

pub fn native_tan(args: &[Value]) -> Result<Value, RuntimeError> {
    unary_number(args, f64::tan)
}

pub fn native_pow(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let base = expect_number(&args[0])?;
    let exponent = expect_number(&args[1])?;

    Ok(Value::Float(base.powf(exponent)))
}

pub fn native_min(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let a = expect_number(&args[0])?;
    let b = expect_number(&args[1])?;

    if a <= b {
        Ok(args[0].clone())
    } else {
        Ok(args[1].clone())
    }
}

pub fn native_max(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let a = expect_number(&args[0])?;
    let b = expect_number(&args[1])?;

    if a >= b {
        Ok(args[0].clone())
    } else {
        Ok(args[1].clone())
    }
}

fn register_one(
    globals: &mut HashMap<String, Value>,
    name: &str,
    function: super::NativeFn,
) {
    globals.insert(
        name.to_string(),
        Value::NativeFunction(function),
    );
}

fn define_one(compiler: &mut Compiler, name: &str) {
    let _ = compiler.define_native(name);
}

pub fn register(globals: &mut HashMap<String, Value>) {
    register_one(globals, "abs", native_abs);
    register_one(globals, "floor", native_floor);
    register_one(globals, "ceil", native_ceil);
    register_one(globals, "round", native_round);
    register_one(globals, "sqrt", native_sqrt);
    register_one(globals, "pow", native_pow);
    register_one(globals, "min", native_min);
    register_one(globals, "max", native_max);
    register_one(globals, "sin", native_sin);
    register_one(globals, "cos", native_cos);
    register_one(globals, "tan", native_tan);
    register_one(globals, "log", native_log);
    register_one(globals, "log10", native_log10);
    register_one(globals, "exp", native_exp);
}

pub fn register_compiler(compiler: &mut Compiler) {
    define_one(compiler, "abs");
    define_one(compiler, "floor");
    define_one(compiler, "ceil");
    define_one(compiler, "round");
    define_one(compiler, "sqrt");
    define_one(compiler, "pow");
    define_one(compiler, "min");
    define_one(compiler, "max");
    define_one(compiler, "sin");
    define_one(compiler, "cos");
    define_one(compiler, "tan");
    define_one(compiler, "log");
    define_one(compiler, "log10");
    define_one(compiler, "exp");
}
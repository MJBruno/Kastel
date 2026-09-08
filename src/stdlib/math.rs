use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static RNG_STATE: AtomicU64 = AtomicU64::new(0);

use crate::{
    compiler::compiler::Compiler, error::runtime_error::RuntimeError, runtime::value::Value,
};

fn expect_number(value: &Value) -> Result<f64, RuntimeError> {
    match value {
        Value::Integer(value) => Ok(*value as f64),
        Value::Float(value) => Ok(*value),
        _ => Err(RuntimeError::TypeError),
    }
}

fn unary_number(args: &[Value], operation: fn(f64) -> f64) -> Result<Value, RuntimeError> {
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

fn next_random_u64() -> u64 {
    let mut state = RNG_STATE.load(Ordering::Relaxed);

    if state == 0 {
        state = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15);

        if state == 0 {
            state = 0x9E37_79B9_7F4A_7C15;
        }
    }

    // xorshift64*
    state ^= state >> 12;
    state ^= state << 25;
    state ^= state >> 27;

    let result = state.wrapping_mul(0x2545_F491_4F6C_DD1D);

    RNG_STATE.store(state, Ordering::Relaxed);

    result
}

fn random_unit() -> f64 {
    let value = next_random_u64();

    // 53 bits de précision, comme les générateurs flottants classiques.
    ((value >> 11) as f64) * (1.0 / 9_007_199_254_740_992.0)
}

pub fn native_rand(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 0,
            found: args.len(),
        });
    }

    Ok(Value::Float(random_unit()))
}

pub fn native_rand_int(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let max = match args[0] {
        Value::Integer(value) => value,

        Value::Float(value)
            if value.is_finite()
                && value.fract() == 0.0
                && (0.0..9_223_372_036_854_775_808.0).contains(&value) =>
        {
            value as i64
        }

        _ => return Err(RuntimeError::TypeError),
    };

    if max <= 0 {
        return Err(RuntimeError::TypeError);
    }

    let max_u64 = max as u64;

    let threshold = u64::MAX - (u64::MAX % max_u64);

    loop {
        let value = next_random_u64();

        if value < threshold {
            return Ok(Value::Integer((value % max_u64) as i64));
        }
    }
}

pub fn native_rand_range(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let start = match args[0] {
        Value::Integer(value) => value,

        Value::Float(value)
            if value.is_finite()
                && value.fract() == 0.0
                && value >= i64::MIN as f64
                && value < 9_223_372_036_854_775_808.0 =>
        {
            value as i64
        }

        _ => return Err(RuntimeError::TypeError),
    };

    let end = match args[1] {
        Value::Integer(value) => value,

        Value::Float(value)
            if value.is_finite()
                && value.fract() == 0.0
                && value >= i64::MIN as f64
                && value < 9_223_372_036_854_775_808.0 =>
        {
            value as i64
        }

        _ => return Err(RuntimeError::TypeError),
    };

    if start >= end {
        return Err(RuntimeError::TypeError);
    }

    let range = (end as i128) - (start as i128);

    let range_u64 = u64::try_from(range).map_err(|_| RuntimeError::TypeError)?;

    let threshold = u64::MAX - (u64::MAX % range_u64);

    loop {
        let value = next_random_u64();

        if value < threshold {
            let offset = (value % range_u64) as i128;
            let result = (start as i128) + offset;

            return Ok(Value::Integer(result as i64));
        }
    }
}

fn register_one(globals: &mut HashMap<String, Value>, name: &str, function: super::NativeFn) {
    globals.insert(name.to_string(), Value::NativeFunction(function));
}

fn define_one(compiler: &mut Compiler, name: &str) {
    let _ = compiler.define_native(name);
}

pub fn register(globals: &mut HashMap<String, Value>) {
    register_one(globals, "rand", native_rand);
    register_one(globals, "rand_int", native_rand_int);
    register_one(globals, "rand_range", native_rand_range);
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
    define_one(compiler, "rand");
    define_one(compiler, "rand_int");
    define_one(compiler, "rand_range");
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

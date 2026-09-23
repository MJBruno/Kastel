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

/// Convertit un f64 (résultat de floor/ceil/round) en Integer, comme le
/// font math.floor/math.ceil/round() en Python (qui renvoient un int,
/// pas un float).
fn float_to_integer(value: f64) -> Result<Value, RuntimeError> {
    // NaN n'a pas de valeur entière ; ±infini et les valeurs hors de
    // l'intervalle i64 sont un DÉPASSEMENT (jamais un écrêtage silencieux
    // à i64::MAX). La borne haute est exclusive : `i64::MAX as f64` vaut
    // déjà 2^63, qui n'est pas représentable.
    const I64_MAX_EXCLUSIVE: f64 = 9_223_372_036_854_775_808.0;

    if value.is_nan() {
        return Err(RuntimeError::TypeError);
    }

    if !value.is_finite() || value < i64::MIN as f64 || value >= I64_MAX_EXCLUSIVE {
        return Err(RuntimeError::IntegerOverflow {
            operation: "conversion en entier",
        });
    }

    Ok(Value::Integer(value as i64))
}

fn unary_number_to_integer(
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

    float_to_integer(operation(value))
}

/// round() "à la Python" : arrondi au plus proche, et en cas d'égalité
/// exacte (x.5), arrondi vers le nombre pair le plus proche ("banker's
/// rounding"), contrairement à f64::round() de Rust qui arrondit
/// toujours en s'éloignant de zéro.
fn python_round(value: f64) -> f64 {
    let floor = value.floor();
    let diff = value - floor;

    if diff < 0.5 {
        floor
    } else if diff > 0.5 {
        floor + 1.0
    } else if (floor.rem_euclid(2.0)) == 0.0 {
        floor
    } else {
        floor + 1.0
    }
}

pub fn native_abs(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    match args[0] {
        Value::Integer(value) => {
            value
                .checked_abs()
                .map(Value::Integer)
                .ok_or(RuntimeError::IntegerOverflow {
                    operation: "valeur absolue",
                })
        }

        Value::Float(value) => Ok(Value::Float(value.abs())),

        _ => Err(RuntimeError::TypeError),
    }
}

pub fn native_floor(args: &[Value]) -> Result<Value, RuntimeError> {
    unary_number_to_integer(args, f64::floor)
}

pub fn native_ceil(args: &[Value]) -> Result<Value, RuntimeError> {
    unary_number_to_integer(args, f64::ceil)
}

pub fn native_round(args: &[Value]) -> Result<Value, RuntimeError> {
    unary_number_to_integer(args, python_round)
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

pub fn native_asin(args: &[Value]) -> Result<Value, RuntimeError> {
    unary_number(args, f64::asin)
}

pub fn native_acos(args: &[Value]) -> Result<Value, RuntimeError> {
    unary_number(args, f64::acos)
}

pub fn native_atan(args: &[Value]) -> Result<Value, RuntimeError> {
    unary_number(args, f64::atan)
}

/// atan2(y, x) : arc tangente à deux arguments (angle du vecteur
/// (x, y), correctement défini sur les 4 quadrants, contrairement à
/// atan(y / x) seul qui ne peut pas distinguer (1, 1) de (-1, -1)).
/// Ne peut pas être exprimée en Kastel pur à partir de sin/cos/tan
/// sans perdre la gestion des quadrants/division par zéro — d'où
/// l'ajout de cette native, sur le même modèle que pow().
pub fn native_atan2(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let y = expect_number(&args[0])?;
    let x = expect_number(&args[1])?;

    Ok(Value::Float(y.atan2(x)))
}

pub fn native_pow(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    // Comme pow() en Python : deux entiers avec un exposant positif ou
    // nul donnent un résultat entier (8, pas 8.0). Dès qu'un flottant
    // ou un exposant négatif entre en jeu, le résultat est un Float.
    if let (Value::Integer(base), Value::Integer(exponent)) = (&args[0], &args[1]) {
        if *exponent >= 0 {
            let overflow = RuntimeError::IntegerOverflow {
                operation: "puissance",
            };

            return match u32::try_from(*exponent) {
                Ok(exponent) => base
                    .checked_pow(exponent)
                    .map(Value::Integer)
                    .ok_or(overflow),

                // Exposant > u32::MAX : seuls 0, 1 et -1 restent
                // représentables.
                Err(_) => match *base {
                    0 => Ok(Value::Integer(0)),
                    1 => Ok(Value::Integer(1)),
                    -1 => Ok(Value::Integer(if exponent % 2 == 0 { 1 } else { -1 })),
                    _ => Err(overflow),
                },
            };
        }
    }

    let base = expect_number(&args[0])?;
    let exponent = expect_number(&args[1])?;

    Ok(Value::Float(base.powf(exponent)))
}

// ============================================================
//              ARITHMÉTIQUE CYCLIQUE EXPLICITE
// ============================================================
//
// `+`, `-` et `*` lèvent `IntegerOverflow` quand le résultat sort de 64 bits.
// Ces trois fonctions sont l'exception VOLONTAIRE : le résultat « boucle »
// modulo 2^64 (fonctions de hachage, générateurs pseudo-aléatoires...).

fn wrapping_operation(
    args: &[Value],
    operation: fn(i64, i64) -> i64,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(operation(*a, *b))),
        _ => Err(RuntimeError::TypeError),
    }
}

pub fn native_wrapping_add(args: &[Value]) -> Result<Value, RuntimeError> {
    wrapping_operation(args, i64::wrapping_add)
}

pub fn native_wrapping_sub(args: &[Value]) -> Result<Value, RuntimeError> {
    wrapping_operation(args, i64::wrapping_sub)
}

pub fn native_wrapping_mul(args: &[Value]) -> Result<Value, RuntimeError> {
    wrapping_operation(args, i64::wrapping_mul)
}

/// `idiv(a, b)` : division ENTIÈRE arrondie vers -infini (`idiv(7, 2)` = 3,
/// `idiv(-7, 2)` = -4). Exacte sur 64 bits, contrairement à `floor(a / b)` :
/// `/` passe par un flottant, donc perd de la précision au-delà de 2^53.
pub fn native_idiv(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    if let (Value::Integer(a), Value::Integer(b)) = (&args[0], &args[1]) {
        let overflow = RuntimeError::IntegerOverflow {
            operation: "division entière",
        };

        if *b == 0 {
            return Err(RuntimeError::DivisionByZero);
        }

        // Seul `i64::MIN / -1` déborde.
        let quotient = a.checked_div(*b).ok_or(overflow)?;

        // `checked_div` a écarté MIN / -1 : le reste est sûr.
        let remainder = *a % *b;

        return if remainder != 0 && ((*a < 0) != (*b < 0)) {
            quotient
                .checked_sub(1)
                .map(Value::Integer)
                .ok_or(RuntimeError::IntegerOverflow {
                    operation: "division entière",
                })
        } else {
            Ok(Value::Integer(quotient))
        };
    }

    let a = expect_number(&args[0])?;
    let b = expect_number(&args[1])?;

    if b == 0.0 {
        return Err(RuntimeError::DivisionByZero);
    }

    float_to_integer((a / b).floor())
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
    register_one(globals, "asin", native_asin);
    register_one(globals, "acos", native_acos);
    register_one(globals, "atan", native_atan);
    register_one(globals, "atan2", native_atan2);
    register_one(globals, "log", native_log);
    register_one(globals, "log10", native_log10);
    register_one(globals, "exp", native_exp);
    register_one(globals, "wrapping_add", native_wrapping_add);
    register_one(globals, "wrapping_sub", native_wrapping_sub);
    register_one(globals, "wrapping_mul", native_wrapping_mul);
    register_one(globals, "idiv", native_idiv);
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
    define_one(compiler, "asin");
    define_one(compiler, "acos");
    define_one(compiler, "atan");
    define_one(compiler, "atan2");
    define_one(compiler, "log");
    define_one(compiler, "log10");
    define_one(compiler, "exp");
    define_one(compiler, "wrapping_add");
    define_one(compiler, "wrapping_sub");
    define_one(compiler, "wrapping_mul");
    define_one(compiler, "idiv");
}

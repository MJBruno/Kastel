use std::collections::HashMap;

use crate::{
    compiler::compiler::Compiler, error::runtime_error::RuntimeError, runtime::value::Value,
};

/// Convertit une valeur Kastel en entier i64.
///
/// Les floats sont acceptés uniquement s'ils sont :
/// - finis
/// - entiers mathématiquement
/// - représentables dans l'intervalle i64
fn expect_integer(value: &Value) -> Result<i64, RuntimeError> {
    match value {
        Value::Integer(value) => Ok(*value),

        Value::Float(value)
            if value.is_finite()
                && value.fract() == 0.0
                && *value >= i64::MIN as f64
                && *value < 9_223_372_036_854_775_808.0 =>
        {
            Ok(*value as i64)
        }

        _ => Err(RuntimeError::TypeError),
    }
}

/// Crée un range paresseux.
///
/// range(stop)
/// range(start, stop)
/// range(start, stop, step)
pub fn native_range(args: &[Value]) -> Result<Value, RuntimeError> {
    let (start, stop, step) = match args.len() {
        1 => {
            let stop = expect_integer(&args[0])?;

            (0, stop, 1)
        }

        2 => {
            let start = expect_integer(&args[0])?;
            let stop = expect_integer(&args[1])?;

            (start, stop, 1)
        }

        3 => {
            let start = expect_integer(&args[0])?;
            let stop = expect_integer(&args[1])?;
            let step = expect_integer(&args[2])?;

            (start, stop, step)
        }

        found => {
            return Err(RuntimeError::WrongArgumentCount { expected: 3, found });
        }
    };

    if step == 0 {
        return Err(RuntimeError::TypeError);
    }

    Ok(Value::new_range(start as f64, stop as f64, step as f64))
}

/// Convertit un objet itérable en tableau.
pub fn native_list(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    crate::runtime::iterator::drain_to_array(&args[0])
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert("range".to_string(), Value::NativeFunction(native_range));

    globals.insert("list".to_string(), Value::NativeFunction(native_list));
}

pub fn register_compiler(compiler: &mut Compiler) {
    let _ = compiler.define_native("range");
    let _ = compiler.define_native("list");
}

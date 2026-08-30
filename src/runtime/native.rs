use crate::{compile::compiler::Compiler, error::runtime_error::RuntimeError, runtime::value::Value};
 

use std::io::{self, Write};
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
// FORMAT STRING (façon Rust : "{}" comme placeholder positionnel)
// ================================================================
//
// "Bonjour {}, tu as {} ans" avec ["Alice", 30] -> "Bonjour Alice, tu as 30 ans"
//
// - S'il manque des arguments pour les placeholders présents : erreur claire.
// - S'il y a des arguments en trop (non utilisés par un "{}") : ignorés
//   silencieusement, par permissivité (contrairement à println! en Rust,
//   qui est vérifié à la compilation — ici tout se joue à l'exécution).
fn format_string(format: &str, args: &[Value]) -> Result<String, RuntimeError> {
    let mut result = String::with_capacity(format.len());
    let mut chars = format.chars().peekable();
    let mut arg_index = 0;

    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'}') {
            chars.next(); // consomme le '}'

            let value = args.get(arg_index).ok_or(RuntimeError::WrongArgumentCount {
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

// ================================================================
// PRINT / PRINTLN
//
// println(valeur)                          -> comportement historique, AVEC saut de ligne
// println("Bonjour {}", nom)                -> formatage façon Rust
// print("Bonjour {}", nom)                  -> identique, SANS saut de ligne (comme print! en Rust)
// ================================================================

pub fn native_println(args: &[Value]) -> Result<Value, RuntimeError> {
    let formatted = render_arguments(args)?;

    println!("{formatted}");

    Ok(Value::String(formatted))
}

pub fn native_print(args: &[Value]) -> Result<Value, RuntimeError> {
    let formatted = render_arguments(args)?;

    print!("{formatted}");

    // print! (contrairement à println!) n'envoie pas de \n, qui déclenche
    // habituellement le flush de stdout en mode ligne. Sans flush explicite,
    // le texte peut rester bloqué dans le buffer et ne jamais s'afficher
    // avant, par exemple, un prochain input().
    io::stdout().flush().ok();

    Ok(Value::String(formatted))
}

/// Factorise la logique commune à print() et println() : un seul argument
/// non-chaîne affiché tel quel, ou un premier argument "chaîne de format"
/// avec des placeholders "{}" à combler avec les arguments suivants.
fn render_arguments(args: &[Value]) -> Result<String, RuntimeError> {
    let Some(first) = args.first() else {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: 0,
        });
    };

    if args.len() == 1 {
        if let Value::String(text) = first {
            return Ok(text.clone());
        }

        return Ok(first.to_string());
    }

    let format = match first {
        Value::String(text) => text,
        _ => return Err(RuntimeError::TypeError),
    };

    format_string(format, &args[1..])
}

// ================================================================
// INPUT
//
// input()               -> lit une ligne sur stdin, sans invite
// input("Nom : ")       -> affiche l'invite (sans saut de ligne) puis lit
//
// Le saut de ligne final (et le \r sous Windows) est retiré du résultat.
// ================================================================

pub fn native_input(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() > 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    if let Some(prompt) = args.first() {
        print!("{prompt}");

        io::stdout().flush().map_err(|_| RuntimeError::NativeError)?;
    }

    let mut buffer = String::new();

    io::stdin()
        .read_line(&mut buffer)
        .map_err(|_| RuntimeError::NativeError)?;

    let trimmed = buffer.trim_end_matches(['\n', '\r']).to_string();

    Ok(Value::String(trimmed))
}

// ================================================================
// FORMAT
//
// format("Bonjour {}", nom) -> retourne la chaîne formatée SANS l'afficher
// (équivalent de format! en Rust, à opposer à println! / native_print)
// ================================================================

pub fn native_format(args: &[Value]) -> Result<Value, RuntimeError> {
    let Some(first) = args.first() else {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: 0,
        });
    };

    let format = match first {
        Value::String(text) => text,
        _ => return Err(RuntimeError::TypeError),
    };

    let formatted = format_string(format, &args[1..])?;

    Ok(Value::String(formatted))
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

    globals.insert("println".to_string(), Value::NativeFunction(native_println));

    globals.insert("print".to_string(), Value::NativeFunction(native_print));

    globals.insert("input".to_string(), Value::NativeFunction(native_input));

    globals.insert("format".to_string(), Value::NativeFunction(native_format));

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

    let _ = compiler.define_native("print");

    let _ = compiler.define_native("input");

    let _ = compiler.define_native("format");

    let _ = compiler.define_native("push");

    let _ = compiler.define_native("pop");

    let _ = compiler.define_native("length");

    let _ = compiler.define_native("insert");
    
    let _ = compiler.define_native("remove");
}
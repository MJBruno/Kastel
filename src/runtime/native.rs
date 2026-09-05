use crate::{
    compiler::compiler::Compiler,
    error::runtime_error::RuntimeError,
    runtime::object::Object,
    runtime::value::{NumericOp, Value},
};

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

    Ok(Value::Float(duration.as_secs_f64()))
}

// ================================================================
// RANGE (façon Python)
//
// range(stop)               -> 0, 1, ..., stop-1
// range(start, stop)        -> start, start+1, ..., stop-1
// range(start, stop, step)  -> start, start+step, ... (step != 0)
//
// range() est PARESSEUX : elle ne construit aucun tableau, quelle que soit
// l'amplitude de l'intervalle. range(1_000_000_000) est en O(1) — elle
// retourne un itérateur (3 f64 : position courante, borne, pas), qui ne
// produit ses valeurs qu'au fur et à mesure qu'on les consomme (typiquement
// via `for i in range(...)`).
//
// Pour obtenir un vrai tableau indexable/mutable, matérialiser
// explicitement avec list() : `let arr = list(range(10));`
// ================================================================

fn expect_integer(value: &Value) -> Result<i64, RuntimeError> {
    match value {
        Value::Integer(n) => Ok(*n),

        Value::Float(n) => {
            if n.fract() != 0.0 {
                return Err(RuntimeError::TypeError);
            }

            Ok(*n as i64)
        }

        _ => Err(RuntimeError::TypeError),
    }
}

pub fn native_range(args: &[Value]) -> Result<Value, RuntimeError> {
    let (start, stop, step) = match args.len() {
        1 => (0i64, expect_integer(&args[0])?, 1i64),

        2 => (expect_integer(&args[0])?, expect_integer(&args[1])?, 1i64),

        3 => (
            expect_integer(&args[0])?,
            expect_integer(&args[1])?,
            expect_integer(&args[2])?,
        ),

        found => {
            return Err(RuntimeError::WrongArgumentCount { expected: 3, found });
        }
    };

    if step == 0 {
        return Err(RuntimeError::TypeError);
    }

    Ok(Value::new_range(start as f64, stop as f64, step as f64))
}

// ================================================================
// LIST (façon Python)
//
// list(x) matérialise n'importe quel itérable (Range paresseux, tableau
// existant) en un vrai tableau, indexable et mutable.
//
//   let arr = list(range(10));   // [0, 1, 2, ..., 9]
//   arr[0] = 100;                // possible : arr est un vrai tableau
//   println(arr.length);         // 10
//
// range(10) seul ne permet ni indexation ni .length — c'est justement le
// compromis d'un itérateur paresseux (aucune allocation tant qu'on n'en a
// pas explicitement besoin).
// ================================================================

pub fn native_list(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    crate::runtime::iterator::drain_to_array(&args[0])
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

    Value::binary_numeric_op(args[0].clone(), args[1].clone(), NumericOp::Add)
}

// ================================================================
// CONVERSIONS DE TYPE (façon Python)
//
// int(x)   : Number -> tronque vers zéro (comme Python, pas floor())
//            String -> parsée comme nombre puis tronquée
//            Bool   -> 1 ou 0
// float(x) : mêmes sources, mais sans troncature
// str(x)   : n'importe quelle valeur -> sa représentation textuelle
//            (réutilise Display, déjà défini pour tous les types)
// bool(x)  : n'importe quelle valeur -> sa "vérité" (réutilise is_truthy,
//            déjà défini pour tous les types — logique déjà centralisée,
//            on ne fait que l'exposer comme fonction appelable)
//
// Note : pas de complex(x) — Kastel n'a qu'un seul type numérique (Number,
// un f64), pas de type nombre complexe. L'ajouter proprement demanderait
// une nouvelle variante Value::Complex avec ses propres opérateurs
// arithmétiques (+, -, *, / définis pour des paires (réel, imaginaire)) :
// un vrai ajout de langage, pas une simple fonction de conversion. Dispo
// si tu veux qu'on le fasse en tant que fonctionnalité à part entière.
// ================================================================

fn parse_number_like(value: &Value) -> Result<f64, RuntimeError> {
    match value {
        Value::Integer(n) => Ok(*n as f64),

        Value::Float(n) => Ok(*n),

        Value::Boolean(b) => Ok(if *b { 1.0 } else { 0.0 }),

        _ => {
            if let Some(s) = value.as_string_value() {
                s.trim().parse::<f64>().map_err(|_| RuntimeError::TypeError)
            } else {
                Err(RuntimeError::TypeError)
            }
        }
    }
}

pub fn native_int(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    // Cas rapide : déjà un entier, rien à convertir.
    if let Value::Integer(n) = &args[0] {
        return Ok(Value::Integer(*n));
    }

    let value = parse_number_like(&args[0])?;

    // Troncature vers zéro, pas floor() : int(-5.7) == -5 en Python, pas -6.
    Ok(Value::Integer(value.trunc() as i64))
}

pub fn native_float(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let value = parse_number_like(&args[0])?;

    Ok(Value::Float(value))
}

pub fn native_str(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    Ok(Value::new_string(args[0].to_string()))
}

pub fn native_bool(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    Ok(Value::Boolean(args[0].is_truthy()))
}

// ================================================================
// TYPE (introspection façon Python)
//
// Kastel n'a qu'un seul type numérique en interne (Number = f64) — pas de
// distinction int/float au niveau de Value. type() la simule en testant
// si la partie décimale est nulle, uniquement pour l'affichage : ça ne
// change rien au comportement des opérations arithmétiques.
// ================================================================

pub fn native_type(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let name = match &args[0] {
        Value::Integer(_) => "int",
        Value::Float(_) => "float",

        Value::Boolean(_) => "bool",
        Value::Nil => "nil",
        Value::Range { .. } => "range",
        Value::NativeFunction(_) => "function",

        Value::Object(handle) => match &*handle.borrow() {
            Object::String(_) => "string",
            Object::Array(_) => "array",
            Object::Dict(_) => "object",
            Object::Function(_) | Object::Closure(_) => "function",
            Object::Iterator(_) => "iterator",
            Object::Module(_) => "module",
        },
    };

    Ok(Value::new_string(name.to_string()))
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

    Ok(Value::new_string(formatted))
}

pub fn native_print(args: &[Value]) -> Result<Value, RuntimeError> {
    let formatted = render_arguments(args)?;

    print!("{formatted}");

    // print! (contrairement à println!) n'envoie pas de \n, qui déclenche
    // habituellement le flush de stdout en mode ligne. Sans flush explicite,
    // le texte peut rester bloqué dans le buffer et ne jamais s'afficher
    // avant, par exemple, un prochain input().
    io::stdout().flush().ok();

    Ok(Value::new_string(formatted))
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
        if let Some(text) = first.as_string_value() {
            return Ok(text);
        }

        return Ok(first.to_string());
    }

    let format = first.as_string_value().ok_or(RuntimeError::TypeError)?;

    format_string(&format, &args[1..])
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

        io::stdout()
            .flush()
            .map_err(|_| RuntimeError::NativeError)?;
    }

    let mut buffer = String::new();

    io::stdin()
        .read_line(&mut buffer)
        .map_err(|_| RuntimeError::NativeError)?;

    let trimmed = buffer.trim_end_matches(['\n', '\r']).to_string();

    Ok(Value::new_string(trimmed))
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

    let format = first.as_string_value().ok_or(RuntimeError::TypeError)?;

    let formatted = format_string(&format, &args[1..])?;

    Ok(Value::new_string(formatted))
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

    Ok(Value::Integer(length as i64))
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

    Ok(Value::Integer(length as i64))
}

pub fn native_array_insert(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 3,
            found: args.len(),
        });
    }

    let index = expect_array_index(&args[1])?;

    let length = args[0].array_insert(index, args[2].clone())?;

    Ok(Value::Integer(length as i64))
}

pub fn native_array_remove(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let index = expect_array_index(&args[1])?;

    args[0].array_remove(index)
}

/// Valide un index de tableau : Integer non-négatif directement, ou Float
/// à valeur entière non-négative (cohérent avec l'indexation `arr[i]`,
/// qui accepte aussi les deux — voir `array_index` côté VM).
fn expect_array_index(value: &Value) -> Result<usize, RuntimeError> {
    match value {
        Value::Integer(n) if *n >= 0 => Ok(*n as usize),

        Value::Float(n) if *n >= 0.0 && n.fract() == 0.0 => Ok(*n as usize),

        _ => Err(RuntimeError::TypeError),
    }
}

// ================================================================
// REGISTER NATIVES
// ================================================================

pub fn register_natives(globals: &mut std::collections::HashMap<String, Value>) {
    globals.insert("clock".to_string(), Value::NativeFunction(native_clock));

    globals.insert("int".to_string(), Value::NativeFunction(native_int));

    globals.insert("float".to_string(), Value::NativeFunction(native_float));

    globals.insert("str".to_string(), Value::NativeFunction(native_str));

    globals.insert("bool".to_string(), Value::NativeFunction(native_bool));

    globals.insert("type".to_string(), Value::NativeFunction(native_type));

    globals.insert("range".to_string(), Value::NativeFunction(native_range));

    globals.insert("list".to_string(), Value::NativeFunction(native_list));

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

    let _ = compiler.define_native("int");

    let _ = compiler.define_native("float");

    let _ = compiler.define_native("str");

    let _ = compiler.define_native("bool");

    let _ = compiler.define_native("type");

    let _ = compiler.define_native("range");

    let _ = compiler.define_native("list");

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

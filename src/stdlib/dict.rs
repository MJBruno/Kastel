use std::collections::HashMap;

use crate::{
    compiler::compiler::Compiler,
    error::runtime_error::RuntimeError,
    runtime::object::Object,
    runtime::value::Value,
};

// ============================================================
//                         DICT()
// ============================================================

pub fn native_dict(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 0,
            found: args.len(),
        });
    }

    Ok(Value::new_dict(Vec::new()))
}

// ============================================================
//                         GET
// ============================================================

pub fn native_dict_get(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    args[0].dict_get(&args[1])
}

// ============================================================
//                         SET
// ============================================================

pub fn native_dict_set(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 3,
            found: args.len(),
        });
    }

    args[0].dict_set(&args[1], args[2].clone())?;

    Ok(Value::Nil)
}

// ============================================================
//                         HAS
// ============================================================

pub fn native_dict_has(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    Ok(Value::Boolean(
        args[0].dict_contains(&args[1])?,
    ))
}

// ============================================================
//                        REMOVE
// ============================================================

pub fn native_dict_remove(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    args[0].dict_remove(&args[1])
}

// ============================================================
//                         LENGTH
// ============================================================

pub fn native_dict_length(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    Ok(Value::Integer(args[0].dict_len()? as i64))
}

// ============================================================
//                          KEYS
// ============================================================

pub fn native_dict_keys(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    args[0].dict_keys()
}

// ============================================================
//                         VALUES
// ============================================================

pub fn native_dict_values(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    args[0].dict_values()
}

// ============================================================
//                          ITEMS
// ============================================================

pub fn native_dict_items(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    args[0].dict_items()
}

// ============================================================
//                          CLEAR
// ============================================================

pub fn native_dict_clear(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    args[0].dict_clear()?;

    Ok(Value::Nil)
}

// ============================================================
//                         GET_OR
// ============================================================
//
// dict.get_or(key, default)
//
// Retourne la valeur associée à key si elle existe.
// Sinon retourne default.
//
// ============================================================

pub fn native_dict_get_or(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 3,
            found: args.len(),
        });
    }

    if args[0].dict_contains(&args[1])? {
        args[0].dict_get(&args[1])
    } else {
        Ok(args[2].clone())
    }
}

// ============================================================
//                          UPDATE
// ============================================================
//
// dict.update(other)
//
// Met à jour le dictionnaire courant avec les entrées de other.
//
// Les clés existantes sont remplacées.
// Les nouvelles clés sont ajoutées à la fin.
//
// ============================================================

pub fn native_dict_update(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let entries = match &args[1] {
        Value::Object(handle) => {
            let object = handle.borrow();

            match &*object {
                Object::Dict(entries) => entries.clone(),
                _ => return Err(RuntimeError::TypeError),
            }
        }

        _ => return Err(RuntimeError::TypeError),
    };

    for (key, value) in entries {
        args[0].dict_set(&key, value)?;
    }

    Ok(Value::Nil)
}

// ============================================================
//                           COPY
// ============================================================
//
// dict.copy()
//
// Copie superficielle :
// - nouvelles paires clé/valeur
// - les Value internes sont clonées
// - les objets imbriqués restent partagés
//
// ============================================================

pub fn native_dict_copy(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let entries = match &args[0] {
        Value::Object(handle) => {
            let object = handle.borrow();

            match &*object {
                Object::Dict(entries) => entries.clone(),
                _ => return Err(RuntimeError::TypeError),
            }
        }

        _ => return Err(RuntimeError::TypeError),
    };

    Ok(Value::new_dict(entries))
}

// ============================================================
//                      METHOD DISPATCH
// ============================================================
//
// Le premier élément de args est toujours le receiver.
//
// Exemple :
//
//     dict.get("name")
//
// devient :
//
//     dispatch_method("get", &[dict, "name"])
//
// ============================================================

pub fn dispatch_method(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, RuntimeError> {
    let result = match name {
        "length" => Some(native_dict_length(args)?),

        "get" => Some(native_dict_get(args)?),

        "set" => Some(native_dict_set(args)?),

        "has" => Some(native_dict_has(args)?),

        "remove" => Some(native_dict_remove(args)?),

        "keys" => Some(native_dict_keys(args)?),

        "values" => Some(native_dict_values(args)?),

        "items" => Some(native_dict_items(args)?),

        "clear" => Some(native_dict_clear(args)?),

        "get_or" => Some(native_dict_get_or(args)?),

        "update" => Some(native_dict_update(args)?),

        "copy" => Some(native_dict_copy(args)?),

        _ => return Ok(None),
    };

    Ok(result)
}

// ============================================================
//                     GLOBAL REGISTRATION
// ============================================================

pub fn register(globals: &mut HashMap<String, Value>) {
    // Construction du dictionnaire.
    globals.insert(
        "dict".to_string(),
        Value::NativeFunction(native_dict),
    );

    // Compatibilité avec l'ancienne API globale.
    globals.insert(
        "dict_get".to_string(),
        Value::NativeFunction(native_dict_get),
    );

    globals.insert(
        "dict_set".to_string(),
        Value::NativeFunction(native_dict_set),
    );

    globals.insert(
        "dict_has".to_string(),
        Value::NativeFunction(native_dict_has),
    );

    globals.insert(
        "dict_remove".to_string(),
        Value::NativeFunction(native_dict_remove),
    );

    globals.insert(
        "dict_keys".to_string(),
        Value::NativeFunction(native_dict_keys),
    );

    globals.insert(
        "dict_values".to_string(),
        Value::NativeFunction(native_dict_values),
    );

    globals.insert(
        "dict_items".to_string(),
        Value::NativeFunction(native_dict_items),
    );

    globals.insert(
        "dict_clear".to_string(),
        Value::NativeFunction(native_dict_clear),
    );
}

// ============================================================
//                  COMPILER NATIVE REGISTRATION
// ============================================================

pub fn register_compiler(compiler: &mut Compiler) {
    let _ = compiler.define_native("dict");

    // Compatibilité avec l'ancienne API globale.
    let _ = compiler.define_native("dict_get");
    let _ = compiler.define_native("dict_set");
    let _ = compiler.define_native("dict_has");
    let _ = compiler.define_native("dict_remove");
    let _ = compiler.define_native("dict_keys");
    let _ = compiler.define_native("dict_values");
    let _ = compiler.define_native("dict_items");
    let _ = compiler.define_native("dict_clear");
}
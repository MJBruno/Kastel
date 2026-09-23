use std::collections::HashMap;

use crate::{
    compiler::compiler::Compiler, error::runtime_error::RuntimeError, runtime::value::Value,
};

/// Signature commune de toutes les fonctions natives Kastel.
pub type NativeFn = fn(&[Value]) -> Result<Value, RuntimeError>;

// ============================================================
//                 API STANDARD DES COLLECTIONS
// ============================================================
//
// Une seule convention pour Array, Dict, Tuple, Set, String et Range :
//
//   TAILLE       size()  is_empty()
//   RECHERCHE    contains(x)  index_of(x)
//   MUTATION     add(x)  remove(x)  clear()        (collections mutables)
//   COPIE        copy()                             (collections mutables)
//   CONVERSION   to_list()  to_string()
//   PARCOURS     iter()   (`for x in c` reste la syntaxe principale)
//
// `length()`, `push()`, `has()`, `items()` et `to_iterator()` n'existent
// plus : ils échouent avec « Vouliez-vous dire '…' ? ».

/// Erreur guidée pour une méthode qui a été renommée.
pub(crate) fn renamed_method_error(name: &str, replacement: &str) -> RuntimeError {
    RuntimeError::ObjectFieldNotFound {
        name: name.to_string(),
        suggestion: Some(replacement.to_string()),
    }
}

/// `to_string()` commun à toutes les collections : même rendu que
/// `println` (`[1, 2]`, `{"a": 1}`, `(1, 2)`, `{1, 2}`...).
pub(crate) fn to_string_method(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    Ok(Value::new_string(args[0].to_string()))
}

pub mod array;
pub mod debug;
pub mod dict;
pub mod file;
pub mod io;
pub mod iterator;
pub mod json;
pub mod math;
pub mod os;
pub mod path;
pub mod record;
pub mod set;
pub mod string;
pub mod system;
pub mod tuple;

/// Enregistre toutes les fonctions natives dans les globals du runtime.
pub fn register_natives(globals: &mut HashMap<String, Value>) {
    io::register(globals);
    math::register(globals);
    string::register(globals);
    array::register(globals);
    tuple::register(globals);
    set::register(globals);
    dict::register(globals);
    // object::register(globals);
    iterator::register(globals);
    system::register(globals);
    debug::register(globals);
    json::register(globals);
    file::register(globals);
    path::register(globals);
    os::register(globals);
}

/// Enregistre les natives connues du compilateur.
pub fn register_compiler_natives(compiler: &mut Compiler) {
    io::register_compiler(compiler);
    math::register_compiler(compiler);
    string::register_compiler(compiler);
    array::register_compiler(compiler);
    tuple::register_compiler(compiler);
    set::register_compiler(compiler);
    dict::register_compiler(compiler);
    // object::register_compiler(compiler);
    iterator::register_compiler(compiler);
    system::register_compiler(compiler);
    debug::register_compiler(compiler);
    json::register_compiler(compiler);
    file::register_compiler(compiler);
    path::register_compiler(compiler);
    os::register_compiler(compiler);
}

/// Compatibilité avec l'ancien appel.
pub fn execute_native(compiler: &mut Compiler) {
    register_compiler_natives(compiler);
}

#[cfg(test)]
mod standard_api_tests {
    use super::*;
    use crate::runtime::object::Object;

    fn ints(values: &[i64]) -> Vec<Value> {
        values.iter().map(|value| Value::Integer(*value)).collect()
    }

    fn text(value: &str) -> Value {
        Value::new_string(value.to_string())
    }

    fn array(values: &[i64]) -> Value {
        Value::new_array(ints(values))
    }

    fn dict() -> Value {
        Value::new_dict(vec![
            (text("a"), Value::Integer(10)),
            (text("b"), Value::Integer(20)),
        ])
    }

    fn tuple(values: &[i64]) -> Value {
        Value::new_tuple(ints(values))
    }

    fn set(values: &[i64]) -> Value {
        Value::new_set(ints(values))
    }

    /// Appelle `receiver.name(extra...)` sur le dispatcher de son type.
    fn call(receiver: &Value, name: &str, extra: &[Value]) -> Result<Value, RuntimeError> {
        let mut args = vec![receiver.clone()];
        args.extend_from_slice(extra);

        let kind = match receiver {
            Value::Object(handle) => match &*handle.borrow() {
                Object::Array(_) => 0,
                Object::Dict(_) => 1,
                Object::Tuple(_) => 2,
                Object::Set(_) => 3,
                Object::String(_) => 4,
                _ => panic!("type non pris en charge par ce test"),
            },
            _ => panic!("valeur non objet"),
        };

        let result = match kind {
            0 => array::dispatch_method(name, &args)?,
            1 => dict::dispatch_method(name, &args)?,
            2 => tuple::dispatch_method(name, &args)?,
            3 => set::dispatch_method(name, &args)?,
            _ => string::dispatch_method(name, &args)?,
        };

        result.ok_or(RuntimeError::NativeError)
    }

    fn size_of(receiver: &Value) -> i64 {
        match call(receiver, "size", &[]).unwrap() {
            Value::Integer(size) => size,
            other => panic!("size() doit renvoyer un entier, reçu {other:?}"),
        }
    }

    fn truthy(value: Result<Value, RuntimeError>) -> bool {
        matches!(value.unwrap(), Value::Boolean(true))
    }

    #[test]
    fn size_is_the_same_method_everywhere() {
        assert_eq!(size_of(&array(&[1, 2, 3])), 3);
        assert_eq!(size_of(&dict()), 2);
        assert_eq!(size_of(&tuple(&[1, 2, 3])), 3);
        assert_eq!(size_of(&set(&[1, 2, 3])), 3);

        // Chaînes : nombre de CARACTÈRES Unicode, pas d'octets UTF-8.
        assert_eq!(size_of(&text("Hello")), 5);
        assert_eq!(size_of(&text("é")), 1);
        assert_eq!(size_of(&text("😀")), 1);
    }

    #[test]
    fn is_empty_and_contains_are_the_same_everywhere() {
        assert!(truthy(call(&array(&[]), "is_empty", &[])));
        assert!(!truthy(call(&array(&[1]), "is_empty", &[])));
        assert!(truthy(call(&dict(), "contains", &[text("a")])));
        assert!(!truthy(call(&dict(), "contains", &[text("zzz")])));
        assert!(truthy(call(&tuple(&[]), "is_empty", &[])));
        assert!(truthy(call(
            &set(&[1, 2]),
            "contains",
            &[Value::Integer(2)]
        )));
        assert!(truthy(call(
            &array(&[7, 8]),
            "contains",
            &[Value::Integer(8)]
        )));
        assert!(truthy(call(
            &tuple(&[7, 8]),
            "contains",
            &[Value::Integer(8)]
        )));
        assert!(truthy(call(&text("Hello"), "contains", &[text("ll")])));
        assert!(!truthy(call(&text(""), "contains", &[text("x")])));
        assert!(truthy(call(&text(""), "is_empty", &[])));
    }

    #[test]
    fn removed_names_point_to_their_replacement() {
        fn suggestion(result: Result<Value, RuntimeError>) -> Option<String> {
            match result {
                Err(RuntimeError::ObjectFieldNotFound { suggestion, .. }) => suggestion,
                other => panic!("une erreur guidée était attendue, reçu {other:?}"),
            }
        }

        for receiver in [array(&[1]), dict(), tuple(&[1]), text("x")] {
            assert_eq!(
                suggestion(call(&receiver, "length", &[])).as_deref(),
                Some("size()")
            );
        }

        assert_eq!(
            suggestion(call(&array(&[1]), "push", &[Value::Integer(2)])).as_deref(),
            Some("add(value)")
        );
        assert_eq!(
            suggestion(call(&dict(), "has", &[text("a")])).as_deref(),
            Some("contains(key)")
        );
        assert_eq!(
            suggestion(call(&dict(), "items", &[])).as_deref(),
            Some("entries()")
        );
    }

    #[test]
    fn array_add_remove_and_remove_at() {
        let a = array(&[10, 20, 30]);

        // add() : ajout en fin, doublons permis.
        assert!(truthy(call(&a, "add", &[Value::Integer(20)])));
        assert_eq!(size_of(&a), 4);

        // remove(valeur) : première occurrence, `false` si absente.
        assert!(truthy(call(&a, "remove", &[Value::Integer(20)])));
        assert_eq!(size_of(&a), 3);
        assert!(!truthy(call(&a, "remove", &[Value::Integer(999)])));
        assert_eq!(size_of(&a), 3);

        // remove_at(position) : renvoie l'élément retiré.
        let removed = call(&a, "remove_at", &[Value::Integer(0)]).unwrap();
        assert!(matches!(removed, Value::Integer(10)));
        assert_eq!(size_of(&a), 2);

        // clear() / copy() indépendants.
        let copy = call(&a, "copy", &[]).unwrap();
        call(&a, "clear", &[]).unwrap();
        assert_eq!(size_of(&a), 0);
        assert_eq!(size_of(&copy), 2);
    }

    #[test]
    fn dict_entries_and_copy_clear() {
        let d = dict();

        let entries = call(&d, "entries", &[]).unwrap();
        assert_eq!(size_of(&entries), 2);

        let copy = call(&d, "copy", &[]).unwrap();
        call(&d, "clear", &[]).unwrap();
        assert!(truthy(call(&d, "is_empty", &[])));
        assert_eq!(size_of(&copy), 2);
    }

    #[test]
    fn a_tuple_has_no_mutating_methods() {
        let t = tuple(&[1, 2, 3]);

        for name in ["add", "remove", "clear", "copy"] {
            assert!(
                matches!(tuple::dispatch_method(name, &[t.clone()]), Ok(None)),
                "un tuple ne doit pas exposer {name}()"
            );
        }
    }

    #[test]
    fn to_string_matches_the_println_rendering() {
        let shown = |value: &Value| call(value, "to_string", &[]).unwrap().to_string();

        assert_eq!(shown(&array(&[1, 2])), "[1, 2]");
        assert_eq!(shown(&dict()), "{\"a\": 10, \"b\": 20}");
        assert_eq!(shown(&tuple(&[1, 2])), "(1, 2)");
        assert_eq!(shown(&set(&[1, 2])), "{1, 2}");
        assert_eq!(shown(&text("abc")), "abc");
    }
}

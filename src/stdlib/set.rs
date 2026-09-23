//! Ensemble (`Set`) : éléments uniques, mutable, sans ordre garanti.
//!
//! ```text
//! let s = Set(1, 2, 3);     // ou : let s = {1, 2, 3};
//! s.add(4);
//! s.contains(2);            // true
//! ```
//!
//! Règle de partage identique à Array et Dict : `let b = a;` désigne le
//! MÊME ensemble ; `a.copy()` en fabrique un nouveau.

use std::collections::HashMap;

use crate::{
    compiler::compiler::Compiler, error::runtime_error::RuntimeError, runtime::object::Object,
    runtime::value::Value,
};

fn expect_args(args: &[Value], expected: usize) -> Result<(), RuntimeError> {
    if args.len() != expected {
        return Err(RuntimeError::WrongArgumentCount {
            expected,
            found: args.len(),
        });
    }

    Ok(())
}

/// Éléments d'un argument « collection » : un ensemble, un tableau ou un
/// tuple (`a.union([4, 5])` est accepté).
fn elements_of(value: &Value) -> Result<Vec<Value>, RuntimeError> {
    match value {
        Value::Object(handle) => match &*handle.borrow() {
            Object::Array(elements) | Object::Tuple(elements) => Ok(elements.clone()),
            Object::Set(elements) => Ok(elements.to_vec()),

            _ => Err(RuntimeError::TypeError),
        },

        _ => Err(RuntimeError::TypeError),
    }
}

fn contains(elements: &[Value], value: &Value) -> bool {
    elements
        .iter()
        .any(|element| Value::set_equals(element, value))
}

// ============================================================
//                         SET(...)
// ============================================================

/// `Set(a, b, c)` : les doublons sont éliminés.
pub fn native_set(args: &[Value]) -> Result<Value, RuntimeError> {
    Ok(Value::new_set(args.to_vec()))
}

// ============================================================
//                   OPÉRATIONS DE BASE
// ============================================================

/// `add(value)` -> `true` si l'élément est nouveau.
pub fn native_add(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_args(args, 2)?;

    Ok(Value::Boolean(args[0].set_add(args[1].clone())?))
}

/// `remove(value)` -> `true` si l'élément était présent (jamais d'erreur
/// pour un élément absent).
pub fn native_remove(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_args(args, 2)?;

    Ok(Value::Boolean(args[0].set_remove(&args[1])?))
}

pub fn native_contains(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_args(args, 2)?;

    Ok(Value::Boolean(args[0].set_contains(&args[1])?))
}

pub fn native_size(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_args(args, 1)?;

    Ok(Value::Integer(args[0].set_len()? as i64))
}

pub fn native_is_empty(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_args(args, 1)?;

    Ok(Value::Boolean(args[0].set_len()? == 0))
}

pub fn native_clear(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_args(args, 1)?;

    args[0].set_clear()?;

    Ok(Value::None)
}

/// Copie superficielle : un NOUVEL ensemble, indépendant de l'original.
pub fn native_copy(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_args(args, 1)?;

    Ok(Value::new_set_unchecked(args[0].set_elements()?))
}

pub fn native_to_list(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_args(args, 1)?;

    Ok(Value::new_array(args[0].set_elements()?))
}

// ============================================================
//                   OPÉRATIONS ENSEMBLISTES
// ============================================================
//
// Elles ne modifient NI le receveur NI l'argument : elles renvoient un
// nouvel ensemble.

/// `a.union(b)` : éléments de `a` ou de `b`.
pub fn native_union(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_args(args, 2)?;

    let mut elements = args[0].set_elements()?;
    elements.extend(elements_of(&args[1])?);

    // `new_set` dédoublonne aussi les doublons éventuels de `b`.
    Ok(Value::new_set(elements))
}

/// `a.intersection(b)` : éléments présents dans `a` ET dans `b`.
pub fn native_intersection(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_args(args, 2)?;

    let other = elements_of(&args[1])?;
    let elements = args[0]
        .set_elements()?
        .into_iter()
        .filter(|element| contains(&other, element))
        .collect();

    Ok(Value::new_set_unchecked(elements))
}

/// `a.difference(b)` : éléments de `a` absents de `b`.
pub fn native_difference(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_args(args, 2)?;

    let other = elements_of(&args[1])?;
    let elements = args[0]
        .set_elements()?
        .into_iter()
        .filter(|element| !contains(&other, element))
        .collect();

    Ok(Value::new_set_unchecked(elements))
}

/// `a.symmetric_difference(b)` : éléments présents dans un seul des deux.
pub fn native_symmetric_difference(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_args(args, 2)?;

    let left = args[0].set_elements()?;
    let right = elements_of(&args[1])?;

    let mut elements: Vec<Value> = left
        .iter()
        .filter(|element| !contains(&right, element))
        .cloned()
        .collect();

    elements.extend(
        right
            .iter()
            .filter(|element| !contains(&left, element))
            .cloned(),
    );

    // `right` peut contenir des doublons (tableau) : on dédoublonne.
    Ok(Value::new_set(elements))
}

/// `a.is_subset(b)` : tous les éléments de `a` sont dans `b`.
pub fn native_is_subset(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_args(args, 2)?;

    let other = elements_of(&args[1])?;

    Ok(Value::Boolean(
        args[0]
            .set_elements()?
            .iter()
            .all(|element| contains(&other, element)),
    ))
}

/// `a.is_superset(b)` : tous les éléments de `b` sont dans `a`.
pub fn native_is_superset(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_args(args, 2)?;

    let own = args[0].set_elements()?;

    Ok(Value::Boolean(
        elements_of(&args[1])?
            .iter()
            .all(|element| contains(&own, element)),
    ))
}

/// `a.equals(b)` : mêmes éléments, quel que soit l'ordre. (`==` compare,
/// lui, l'IDENTITÉ des objets, comme pour Array et Dict.)
pub fn native_equals(args: &[Value]) -> Result<Value, RuntimeError> {
    expect_args(args, 2)?;

    let own = args[0].set_elements()?;

    // Comparer un ensemble à autre chose qu'un ensemble n'est jamais vrai.
    let other_is_set = matches!(
        &args[1],
        Value::Object(handle) if matches!(&*handle.borrow(), Object::Set(_))
    );

    if !other_is_set {
        return Ok(Value::Boolean(false));
    }

    let other = elements_of(&args[1])?;

    Ok(Value::Boolean(
        own.len() == other.len() && own.iter().all(|element| contains(&other, element)),
    ))
}

// ============================================================
//                     METHOD DISPATCH
// ============================================================

pub fn dispatch_method(name: &str, args: &[Value]) -> Result<Option<Value>, RuntimeError> {
    let result = match name {
        "add" => native_add(args)?,
        "remove" => native_remove(args)?,
        "contains" => native_contains(args)?,
        "size" => native_size(args)?,
        "is_empty" => native_is_empty(args)?,
        "clear" => native_clear(args)?,
        "copy" => native_copy(args)?,
        "to_list" => native_to_list(args)?,
        "to_array" => return Err(super::renamed_method_error("to_array", "to_list()")),
        "union" => native_union(args)?,
        "intersection" => native_intersection(args)?,
        "difference" => native_difference(args)?,
        "symmetric_difference" => native_symmetric_difference(args)?,
        "is_subset" => native_is_subset(args)?,
        "is_superset" => native_is_superset(args)?,
        "equals" => native_equals(args)?,
        "to_string" => super::to_string_method(args)?,

        _ => return Ok(None),
    };

    Ok(Some(result))
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert("Set".to_string(), Value::NativeFunction(native_set));
}

pub fn register_compiler(compiler: &mut Compiler) {
    let _ = compiler.define_native("Set");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ints(values: &[i64]) -> Value {
        Value::new_set(values.iter().map(|value| Value::Integer(*value)).collect())
    }

    fn size(set: &Value) -> i64 {
        match native_size(&[set.clone()]).unwrap() {
            Value::Integer(size) => size,
            other => panic!("size() doit renvoyer un entier, reçu {other:?}"),
        }
    }

    fn has(set: &Value, value: i64) -> bool {
        set.set_contains(&Value::Integer(value)).unwrap()
    }

    #[test]
    fn set_guarantees_uniqueness() {
        let set = native_set(&[
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(3),
            Value::Integer(3),
        ])
        .unwrap();

        assert_eq!(size(&set), 3);

        // 1 et 1.0 sont le même élément logique (comme pour les clés de dict).
        assert!(!set.set_add(Value::Float(1.0)).unwrap());
        assert_eq!(size(&set), 3);
    }

    #[test]
    fn add_remove_contains_clear() {
        let set = ints(&[1, 2, 3]);

        assert!(set.set_add(Value::Integer(4)).unwrap());
        assert!(!set.set_add(Value::Integer(3)).unwrap());
        assert_eq!(size(&set), 4);
        assert!(has(&set, 2));

        assert!(set.set_remove(&Value::Integer(2)).unwrap());
        assert!(!set.set_remove(&Value::Integer(2)).unwrap());
        assert!(!has(&set, 2));

        set.set_clear().unwrap();
        assert_eq!(size(&set), 0);
        assert!(matches!(
            native_is_empty(&[set]).unwrap(),
            Value::Boolean(true)
        ));
    }

    #[test]
    fn set_operations() {
        let a = ints(&[1, 2, 3]);
        let b = ints(&[3, 4, 5]);

        let union = native_union(&[a.clone(), b.clone()]).unwrap();
        assert_eq!(size(&union), 5);

        let intersection = native_intersection(&[a.clone(), b.clone()]).unwrap();
        assert_eq!(size(&intersection), 1);
        assert!(has(&intersection, 3));

        let difference = native_difference(&[a.clone(), b.clone()]).unwrap();
        assert_eq!(size(&difference), 2);
        assert!(has(&difference, 1) && has(&difference, 2));

        let symmetric = native_symmetric_difference(&[a.clone(), b.clone()]).unwrap();
        assert_eq!(size(&symmetric), 4);
        assert!(!has(&symmetric, 3));

        // Les opérandes ne sont pas modifiés.
        assert_eq!(size(&a), 3);
        assert_eq!(size(&b), 3);
    }

    #[test]
    fn subset_and_superset() {
        let small = ints(&[1, 2]);
        let large = ints(&[1, 2, 3]);

        assert!(matches!(
            native_is_subset(&[small.clone(), large.clone()]).unwrap(),
            Value::Boolean(true)
        ));
        assert!(matches!(
            native_is_superset(&[large.clone(), small.clone()]).unwrap(),
            Value::Boolean(true)
        ));
        assert!(matches!(
            native_is_subset(&[large, small]).unwrap(),
            Value::Boolean(false)
        ));
    }

    #[test]
    fn copy_is_independent_and_assignment_shares() {
        let a = ints(&[1, 2, 3]);

        // `let b = a;` : même objet.
        let shared = a.clone();
        shared.set_add(Value::Integer(4)).unwrap();
        assert!(has(&a, 4));

        // `a.copy()` : nouvel objet.
        let copy = native_copy(&[a.clone()]).unwrap();
        copy.set_add(Value::Integer(99)).unwrap();
        assert!(!has(&a, 99));
        assert!(has(&copy, 99));
    }

    #[test]
    fn tuples_are_compared_by_content_and_a_set_can_contain_itself() {
        let pair = |a, b| Value::new_tuple(vec![Value::Integer(a), Value::Integer(b)]);

        let set = Value::new_set(vec![pair(1, 2), pair(1, 2), pair(2, 1)]);
        assert_eq!(size(&set), 2);

        // `s.add(s)` ne doit pas paniquer (emprunts successifs).
        let s = ints(&[1]);
        assert!(s.set_add(s.clone()).unwrap());
        assert!(!s.set_add(s.clone()).unwrap());
    }

    #[test]
    fn methods_accept_arrays_as_other_operand() {
        let a = ints(&[1, 2]);
        let array = Value::new_array(vec![Value::Integer(2), Value::Integer(2), Value::Integer(3)]);

        let union = native_union(&[a, array]).unwrap();
        assert_eq!(size(&union), 3);
    }
}

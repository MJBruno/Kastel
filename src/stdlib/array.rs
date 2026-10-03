use std::cmp::Ordering;
use std::collections::HashMap;

use crate::{
    error::runtime_error::RuntimeError, runtime::gc_handle::Gc, runtime::object::Object,
    runtime::value::Value,
};

fn expect_index(value: &Value) -> Result<usize, RuntimeError> {
    match value {
        Value::Integer(index) if *index >= 0 => {
            usize::try_from(*index).map_err(|_| RuntimeError::TypeError)
        }

        Value::Float(index)
            if index.is_finite()
                && *index >= 0.0
                && index.fract() == 0.0
                && *index <= usize::MAX as f64 =>
        {
            Ok(*index as usize)
        }

        _ => Err(RuntimeError::ArrayIndexNotInteger),
    }
}

fn expect_slice_index(value: &Value, length: usize) -> Result<isize, RuntimeError> {
    let raw = match value {
        Value::Integer(value) => isize::try_from(*value).map_err(|_| RuntimeError::TypeError)?,

        Value::Float(value)
            if value.is_finite()
                && value.fract() == 0.0
                && *value >= isize::MIN as f64
                && *value <= isize::MAX as f64 =>
        {
            *value as isize
        }

        _ => return Err(RuntimeError::ArrayIndexNotInteger),
    };

    let len = length as isize;

    if raw < 0 {
        Ok((len + raw).max(0))
    } else {
        Ok(raw.min(len))
    }
}

fn with_array<R>(
    value: &Value,
    callback: impl FnOnce(&Vec<Value>) -> Result<R, RuntimeError>,
) -> Result<R, RuntimeError> {
    match value {
        Value::Object(handle) => {
            let object = handle.borrow();

            match &*object {
                Object::Array(array) => callback(array),
                _ => Err(RuntimeError::TypeError),
            }
        }

        _ => Err(RuntimeError::TypeError),
    }
}

fn with_array_mut<R>(
    value: &Value,
    callback: impl FnOnce(&mut Vec<Value>) -> Result<R, RuntimeError>,
) -> Result<R, RuntimeError> {
    match value {
        Value::Object(handle) => {
            let mut object = handle.borrow_mut();

            match &mut *object {
                Object::Array(array) => callback(array),
                _ => Err(RuntimeError::TypeError),
            }
        }

        _ => Err(RuntimeError::TypeError),
    }
}
#[allow(dead_code)]
pub fn native_dict_compat_array_placeholder(_args: &[Value]) -> Result<Value, RuntimeError> {
    Err(RuntimeError::NativeError)
}

// ============================================================
//                      LIST METHODS
// ============================================================
pub fn native_size(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    Ok(Value::Integer(with_array(&args[0], |array| {
        Ok(array.len() as i64)
    })?))
}

pub fn native_is_empty(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    Ok(Value::Boolean(with_array(&args[0], |array| {
        Ok(array.is_empty())
    })?))
}

/// `add(value)` : ajoute en fin de tableau. Renvoie toujours `true`
/// (même signature que `Set.add`, qui renvoie `false` pour un doublon).
pub fn native_add(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    with_array_mut(&args[0], |array| {
        array.push(args[1].clone());
        Ok(())
    })?;

    Ok(Value::Boolean(true))
}

pub fn native_pop(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    with_array_mut(&args[0], |array| Ok(array.pop().unwrap_or(Value::None)))
}

pub fn native_insert(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 3,
            found: args.len(),
        });
    }

    let index = expect_index(&args[1])?;

    let length = with_array_mut(&args[0], |array| {
        if index > array.len() {
            return Err(RuntimeError::ArrayIndexOutOfBounds {
                index,
                length: array.len(),
            });
        }

        array.insert(index, args[2].clone());

        Ok(array.len())
    })?;

    Ok(Value::Integer(length as i64))
}

/// `remove(value)` : retire la PREMIÈRE occurrence de `value`. Renvoie
/// `true` si elle était présente (jamais d'erreur pour une valeur absente),
/// comme `Set.remove`. Pour retirer par POSITION : `remove_at(index)`.
pub fn native_remove(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let position = with_array(&args[0], |array| {
        Ok(array
            .iter()
            .position(|element| Value::equals(element.clone(), args[1].clone())))
    })?;

    match position {
        Some(index) => with_array_mut(&args[0], |array| {
            array.remove(index);
            Ok(Value::Boolean(true))
        }),

        None => Ok(Value::Boolean(false)),
    }
}

/// `remove_at(index)` : retire l'élément à la position `index` et le
/// renvoie (ancien comportement de `remove(index)`).
pub fn native_remove_at(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let index = expect_index(&args[1])?;

    with_array_mut(&args[0], |array| {
        if index >= array.len() {
            return Err(RuntimeError::ArrayIndexOutOfBounds {
                index,
                length: array.len(),
            });
        }

        Ok(array.remove(index))
    })
}

pub fn native_get(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let index = expect_index(&args[1])?;

    with_array(&args[0], |array| {
        array
            .get(index)
            .cloned()
            .ok_or(RuntimeError::ArrayIndexOutOfBounds {
                index,
                length: array.len(),
            })
    })
}

pub fn native_set(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 3,
            found: args.len(),
        });
    }

    let index = expect_index(&args[1])?;

    with_array_mut(&args[0], |array| {
        if index >= array.len() {
            return Err(RuntimeError::ArrayIndexOutOfBounds {
                index,
                length: array.len(),
            });
        }

        array[index] = args[2].clone();

        Ok(Value::None)
    })
}

pub fn native_contains(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    with_array(&args[0], |array| {
        Ok(Value::Boolean(array.iter().any(|value| {
            Value::equals(value.clone(), args[1].clone())
        })))
    })
}

pub fn native_index_of(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    with_array(&args[0], |array| {
        let index = array
            .iter()
            .position(|value| Value::equals(value.clone(), args[1].clone()))
            .map(|index| index as i64)
            .unwrap_or(-1);

        Ok(Value::Integer(index))
    })
}

pub fn native_slice(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 3,
            found: args.len(),
        });
    }

    with_array(&args[0], |array| {
        let start = expect_slice_index(&args[1], array.len())?;

        let end = expect_slice_index(&args[2], array.len())?;

        let start = start as usize;
        let end = end as usize;

        if start >= end {
            return Ok(Value::new_array(Vec::new()));
        }

        Ok(Value::new_array(array[start..end].to_vec()))
    })
}

pub fn native_reverse(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    with_array_mut(&args[0], |array| {
        array.reverse();
        Ok(args[0].clone())
    })
}

pub fn native_join(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let separator = args[1].as_string_value().ok_or(RuntimeError::TypeError)?;

    with_array(&args[0], |array| {
        let mut output = String::new();

        for (index, value) in array.iter().enumerate() {
            if index != 0 {
                output.push_str(&separator);
            }

            output.push_str(&value.to_string());
        }

        Ok(Value::new_string(output))
    })
}

pub fn native_clear(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    with_array_mut(&args[0], |array| {
        array.clear();
        Ok(Value::None)
    })
}

pub fn native_first(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    with_array(&args[0], |array| {
        Ok(array.first().cloned().unwrap_or(Value::None))
    })
}

pub fn native_last(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    with_array(&args[0], |array| {
        Ok(array.last().cloned().unwrap_or(Value::None))
    })
}

pub fn native_sort(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let Value::Object(array_handle) = &args[0] else {
        return Err(RuntimeError::TypeError);
    };

    with_array_mut(&args[0], |array| {
        sort_values(array, array_handle)?;
        Ok(args[0].clone())
    })
}

/// Famille de valeurs triables : on ne trie que des nombres entre eux, des
/// chaînes entre elles ou des booléens entre eux.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SortKind {
    Number,
    Text,
    Boolean,
}

fn sort_kind(value: &Value, array_handle: &Gc<Object>) -> Option<SortKind> {
    match value {
        Value::Integer(_) | Value::Float(_) => Some(SortKind::Number),

        Value::Boolean(_) => Some(SortKind::Boolean),

        Value::Object(handle) => {
            // Le tableau est déjà emprunté en écriture par l'appelant : un
            // tableau qui se contient lui-même ne doit pas être ré-emprunté
            // (panique `already mutably borrowed`).
            if Gc::<Object>::ptr_eq(handle, array_handle) {
                return None;
            }

            match &*handle.borrow() {
                Object::String(_) => Some(SortKind::Text),
                _ => None,
            }
        }

        _ => None,
    }
}

/// Tri en place. Avant ce correctif, `compare_values` renvoyait `Equal` pour
/// toute paire non comparable (types mélangés, objets, NaN) : le tri était
/// alors un no-op SILENCIEUX, et la relation obtenue n'étant pas un ordre
/// total, `sort_by` peut même paniquer (Rust >= 1.81). Désormais :
///   - une liste hétérogène ou non triable lève `TypeError` ;
///   - les nombres suivent un ordre total (comparaison Integer/Float exacte,
///     NaN placé en dernier).
fn sort_values(array: &mut Vec<Value>, array_handle: &Gc<Object>) -> Result<(), RuntimeError> {
    if array.len() < 2 {
        return Ok(());
    }

    let Some(kind) = sort_kind(&array[0], array_handle) else {
        return Err(RuntimeError::TypeError);
    };

    for value in array.iter() {
        if sort_kind(value, array_handle) != Some(kind) {
            return Err(RuntimeError::TypeError);
        }
    }

    match kind {
        SortKind::Number => array.sort_by(compare_numbers),
        SortKind::Boolean => array.sort_by(compare_booleans),
        SortKind::Text => array.sort_by(compare_texts),
    }

    Ok(())
}

fn compare_floats(a: f64, b: f64) -> Ordering {
    match a.partial_cmp(&b) {
        Some(ordering) => ordering,
        // Au moins un NaN : NaN est « plus grand » que tout, et égal à NaN.
        None => a.is_nan().cmp(&b.is_nan()),
    }
}

fn compare_numbers(left: &Value, right: &Value) -> Ordering {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => a.cmp(b),

        (Value::Float(a), Value::Float(b)) => compare_floats(*a, *b),

        (Value::Integer(a), Value::Float(b)) => {
            Value::compare_integer_float(*a, *b).unwrap_or(Ordering::Less)
        }

        (Value::Float(a), Value::Integer(b)) => Value::compare_integer_float(*b, *a)
            .map(Ordering::reverse)
            .unwrap_or(Ordering::Greater),

        _ => Ordering::Equal,
    }
}

fn compare_booleans(left: &Value, right: &Value) -> Ordering {
    match (left, right) {
        (Value::Boolean(a), Value::Boolean(b)) => a.cmp(b),
        _ => Ordering::Equal,
    }
}

fn compare_texts(left: &Value, right: &Value) -> Ordering {
    match (left, right) {
        (Value::Object(a), Value::Object(b)) => match (&*a.borrow(), &*b.borrow()) {
            (Object::String(a), Object::String(b)) => a.cmp(b),
            _ => Ordering::Equal,
        },
        _ => Ordering::Equal,
    }
}

pub fn native_copy(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    with_array(&args[0], |array| Ok(Value::new_array(array.clone())))
}

// ============================================================
//                     METHOD DISPATCH
// ============================================================

pub fn dispatch_method(name: &str, args: &[Value]) -> Result<Option<Value>, RuntimeError> {
    let result = match name {
        // API standard des collections.
        "size" => Some(native_size(args)?),
        "is_empty" => Some(native_is_empty(args)?),
        "add" => Some(native_add(args)?),
        "remove" => Some(native_remove(args)?),
        "remove_at" => Some(native_remove_at(args)?),
        "to_string" => Some(super::to_string_method(args)?),

        // Noms supprimés.
        "length" => return Err(super::renamed_method_error("length", "size()")),
        "push" => return Err(super::renamed_method_error("push", "add(value)")),

        "pop" => Some(native_pop(args)?),
        "insert" => Some(native_insert(args)?),
        "get" => Some(native_get(args)?),
        "set" => Some(native_set(args)?),
        "contains" => Some(native_contains(args)?),
        "index_of" => Some(native_index_of(args)?),
        "slice" => Some(native_slice(args)?),
        "reverse" => Some(native_reverse(args)?),
        "join" => Some(native_join(args)?),
        "clear" => Some(native_clear(args)?),
        "first" => Some(native_first(args)?),
        "last" => Some(native_last(args)?),
        "sort" => Some(native_sort(args)?),
        "copy" => Some(native_copy(args)?),

        // Ces méthodes sont exécutées directement par la VM.
        "map" | "filter" | "reduce" | "any" | "all" => None,

        _ => return Ok(None),
    };

    Ok(result)
}

// Les anciennes natives globales ne sont plus exposées.
// Les listes sont manipulées par leur API objet.

pub fn register(globals: &mut HashMap<String, Value>) {
    let _ = globals;
}


#[cfg(test)]
mod sort_tests {
    use super::native_sort;
    use crate::runtime::object::Object;
    use crate::runtime::value::Value;

    fn ints(values: &[i64]) -> Value {
        Value::new_array(values.iter().map(|v| Value::Integer(*v)).collect())
    }

    fn contents(array: &Value) -> Vec<Value> {
        match array {
            Value::Object(handle) => match &*handle.borrow() {
                Object::Array(items) => items.clone(),
                _ => panic!("tableau attendu"),
            },
            _ => panic!("tableau attendu"),
        }
    }

    #[test]
    fn sorts_integers() {
        let array = ints(&[3, 1, 2]);
        native_sort(&[array.clone()]).unwrap();
        let sorted: Vec<i64> = contents(&array)
            .into_iter()
            .map(|v| match v {
                Value::Integer(i) => i,
                _ => panic!(),
            })
            .collect();
        assert_eq!(sorted, vec![1, 2, 3]);
    }

    #[test]
    fn mixed_types_are_a_type_error_not_a_silent_noop() {
        let array = Value::new_array(vec![Value::Integer(1), Value::new_string("a".to_string())]);
        assert!(native_sort(&[array]).is_err());
    }

    #[test]
    fn non_sortable_objects_are_a_type_error() {
        let a = Value::new_array(vec![]);
        let b = Value::new_array(vec![]);
        assert!(native_sort(&[Value::new_array(vec![a, b])]).is_err());
    }

    #[test]
    fn nan_does_not_panic_and_sorts_last() {
        let array = Value::new_array(vec![
            Value::Float(f64::NAN),
            Value::Integer(2),
            Value::Float(1.5),
            Value::Float(f64::NAN),
            Value::Integer(1),
        ]);
        native_sort(&[array.clone()]).unwrap();
        let items = contents(&array);
        assert!(matches!(items[0], Value::Integer(1)));
        assert!(matches!(items[1], Value::Float(f) if f == 1.5));
        assert!(matches!(items[2], Value::Integer(2)));
        assert!(matches!(items[3], Value::Float(f) if f.is_nan()));
        assert!(matches!(items[4], Value::Float(f) if f.is_nan()));
    }

    #[test]
    fn array_containing_itself_is_rejected_without_panicking() {
        let array = Value::new_array(vec![Value::Integer(1)]);
        if let Value::Object(handle) = &array {
            if let Object::Array(items) = &mut *handle.borrow_mut() {
                items.push(array.clone());
            }
        }
        assert!(native_sort(&[array]).is_err());
    }
}

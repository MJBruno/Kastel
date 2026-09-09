// ================================================================
// ITERATOR
// ================================================================
//
// Un Iterator Kastel est toujours un Object::Iterator.
//
// Sources natives :
//   - Range
//   - Array
//
// Adaptateurs paresseux :
//   - Map
//   - Filter
//   - Take
//   - Skip
//
// `cached` est utilisé par has_next()/peek() pour regarder l'élément
// suivant sans le perdre.
//
// Les callbacks sont exécutés par la VM, car eux seuls ont accès à
// invoke_sync().
// ================================================================

use crate::error::runtime_error::RuntimeError;
use crate::runtime::gc_handle::Gc;
use crate::runtime::object::Object;
use crate::runtime::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum IteratorKind {
    Range {
        current: f64,
        stop: f64,
        step: f64,
    },

    Array {
        array: Gc<Object>,
        index: usize,
    },

    Dict {
        dict: Gc<Object>,
        index: usize,
    },

    String {
        string: Gc<Object>,
        index: usize,
    },

    Map {
        source: Box<Value>,
        callback: Value,
    },

    Filter {
        source: Box<Value>,
        callback: Value,
    },

    Take {
        source: Box<Value>,
        remaining: usize,
    },

    Skip {
        source: Box<Value>,
        remaining: usize,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub struct IteratorState {
    pub(crate) kind: IteratorKind,

    // Élément récupéré par peek()/has_next(), mais pas encore consommé.
    pub(crate) cached: Option<Value>,
}

impl IteratorState {
    pub(crate) fn new(kind: IteratorKind) -> Self {
        Self { kind, cached: None }
    }

    /// Réinitialisation utilisée par le GC pour casser les cycles.
    pub(crate) fn reset_for_gc(&mut self) {
        self.kind = IteratorKind::Range {
            current: 0.0,
            stop: 0.0,
            step: 1.0,
        };

        self.cached = None;
    }

    /// Donne toutes les Value retenues par l'itérateur au GC.
    pub(crate) fn visit_values<F>(&self, mut visit: F)
    where
        F: FnMut(&Value),
    {
        if let Some(cached) = &self.cached {
            visit(cached);
        }

        match &self.kind {
            IteratorKind::Range { .. } => {}

            IteratorKind::Array { .. } => {}

            IteratorKind::Dict { .. } => {}

            IteratorKind::String { .. } => {}
            IteratorKind::Map { source, callback } => {
                visit(source);
                visit(callback);
            }

            IteratorKind::Filter { source, callback } => {
                visit(source);
                visit(callback);
            }

            IteratorKind::Take { source, .. } => {
                visit(source);
            }

            IteratorKind::Skip { source, .. } => {
                visit(source);
            }
        }
    }
}

impl Value {
    // ============================================================
    //                      CONSTRUCTION
    // ============================================================

    /// Range léger et réutilisable.
    pub fn new_range(start: f64, stop: f64, step: f64) -> Self {
        Self::new_range_iterator(start, stop, step)
    }

    fn new_iterator(state: IteratorState) -> Value {
        let handle = Gc::new(Object::Iterator(state));

        crate::runtime::gc::register_object(&handle);

        Value::Object(handle)
    }

    fn new_range_iterator(start: f64, stop: f64, step: f64) -> Value {
        Self::new_iterator(IteratorState::new(IteratorKind::Range {
            current: start,
            stop,
            step,
        }))
    }

    fn new_array_iterator(array: Gc<Object>) -> Value {
        Self::new_iterator(IteratorState::new(IteratorKind::Array { array, index: 0 }))
    }

    // ============================================================
    //                 LAZY ITERATOR CONSTRUCTORS
    // ============================================================

    pub(crate) fn new_map_iterator(source: Value, callback: Value) -> Value {
        Self::new_iterator(IteratorState::new(IteratorKind::Map {
            source: Box::new(source),
            callback,
        }))
    }

    pub(crate) fn new_filter_iterator(source: Value, callback: Value) -> Value {
        Self::new_iterator(IteratorState::new(IteratorKind::Filter {
            source: Box::new(source),
            callback,
        }))
    }

    pub(crate) fn new_take_iterator(source: Value, remaining: usize) -> Value {
        Self::new_iterator(IteratorState::new(IteratorKind::Take {
            source: Box::new(source),
            remaining,
        }))
    }

    pub(crate) fn new_skip_iterator(source: Value, remaining: usize) -> Value {
        Self::new_iterator(IteratorState::new(IteratorKind::Skip {
            source: Box::new(source),
            remaining,
        }))
    }

    // ============================================================
    //                      TO ITERATOR
    // ============================================================

    /// Convertit une valeur en itérateur à état.
    ///
    /// Range  -> nouvel itérateur
    /// Array  -> nouvel itérateur
    /// Iterator -> lui-même
    pub fn to_iterator(&self) -> Result<Value, RuntimeError> {
        match self {
            // ========================================================
            // RANGE
            // ========================================================
            Value::Range { start, stop, step } => {
                Ok(Value::new_range_iterator(*start, *stop, *step))
            }

            // ========================================================
            // OBJECT
            // ========================================================
            Value::Object(handle) => {
                match &*handle.borrow() {
                    // -----------------------------------------------
                    // ITERATOR
                    // -----------------------------------------------
                    Object::Iterator(_) => Ok(self.clone()),

                    // -----------------------------------------------
                    // ARRAY
                    // -----------------------------------------------
                    Object::Array(_) => Ok(Value::new_array_iterator(handle.clone())),

                    // -----------------------------------------------
                    // DICT
                    // -----------------------------------------------
                    Object::Dict(_) => Ok(Value::new_dict_iterator(handle.clone())),

                    // -----------------------------------------------
                    // STRING
                    // -----------------------------------------------
                    Object::String(_) => Ok(Value::new_string_iterator(handle.clone())),

                    _ => Err(RuntimeError::NotIterable),
                }
            }

            _ => Err(RuntimeError::NotIterable),
        }
    }
    fn new_dict_iterator(dict: Gc<Object>) -> Value {
        Self::new_iterator(IteratorState::new(IteratorKind::Dict { dict, index: 0 }))
    }

    fn new_string_iterator(string: Gc<Object>) -> Value {
        Self::new_iterator(IteratorState::new(IteratorKind::String {
            string,
            index: 0,
        }))
    }

    // ============================================================
    //             LEGACY LOW-LEVEL ITERATOR API
    // ============================================================
    //
    // Ces deux méthodes restent pour les utilisateurs internes
    // existants. Les iterators fonctionnels sont exécutés par la VM.
    //
    // ============================================================

    pub fn iterator_has_next(&self) -> Result<bool, RuntimeError> {
        let Value::Object(handle) = self else {
            return Err(RuntimeError::TypeError);
        };

        let object = handle.borrow();

        let Object::Iterator(state) = &*object else {
            return Err(RuntimeError::TypeError);
        };

        if state.cached.is_some() {
            return Ok(true);
        }

        match &state.kind {
            IteratorKind::Range {
                current,
                stop,
                step,
            } => {
                if *step >= 0.0 {
                    Ok(current < stop)
                } else {
                    Ok(current > stop)
                }
            }
            IteratorKind::Dict { dict, index } => {
                let dict = dict.borrow();

                let Object::Dict(entries) = &*dict else {
                    return Err(RuntimeError::TypeError);
                };

                Ok(*index < entries.len())
            }

            IteratorKind::String { string, index } => {
                let string = string.borrow();

                let Object::String(value) = &*string else {
                    return Err(RuntimeError::TypeError);
                };

                Ok(*index < value.chars().count())
            }
            IteratorKind::Array { array, index } => {
                let array = array.borrow();

                let Object::Array(elements) = &*array else {
                    return Err(RuntimeError::TypeError);
                };

                Ok(*index < elements.len())
            }

            // Les adaptateurs avec callback doivent passer par la VM.
            IteratorKind::Map { .. }
            | IteratorKind::Filter { .. }
            | IteratorKind::Take { .. }
            | IteratorKind::Skip { .. } => Err(RuntimeError::TypeError),
        }
    }
    // pub fn is_iterable(value: &Value) -> bool {
    //     match value {
    //         Value::Range { .. } => true,

    //         Value::Object(handle) => {
    //             matches!(
    //                 &*handle.borrow(),
    //                 Object::Array(_) | Object::Dict(_) | Object::String(_) | Object::Iterator(_)
    //             )
    //         }

    //         _ => false,
    //     }
    // }
    pub fn iterator_next(&self) -> Result<Value, RuntimeError> {
        let Value::Object(handle) = self else {
            return Err(RuntimeError::TypeError);
        };

        let mut object = handle.borrow_mut();

        let Object::Iterator(state) = &mut *object else {
            return Err(RuntimeError::TypeError);
        };

        if let Some(value) = state.cached.take() {
            return Ok(value);
        }

        match &mut state.kind {
            // ============================================================
            // RANGE
            // ============================================================
            IteratorKind::Range {
                current,
                stop,
                step,
            } => {
                let has_next = if *step >= 0.0 {
                    *current < *stop
                } else {
                    *current > *stop
                };

                if !has_next {
                    return Err(RuntimeError::IteratorExhausted);
                }

                let value = *current;

                *current += *step;

                Ok(Value::Integer(value as i64))
            }

            // ============================================================
            // ARRAY
            // ============================================================
            IteratorKind::Array { array, index } => {
                let array = array.borrow();

                let Object::Array(elements) = &*array else {
                    return Err(RuntimeError::TypeError);
                };

                if *index >= elements.len() {
                    return Err(RuntimeError::IteratorExhausted);
                }

                let value = elements[*index].clone();

                *index += 1;

                Ok(value)
            }

            // ============================================================
            // DICT
            //
            // dict -> iterator des clés
            // ============================================================
            IteratorKind::Dict { dict, index } => {
                let dict = dict.borrow();

                let Object::Dict(entries) = &*dict else {
                    return Err(RuntimeError::TypeError);
                };

                if *index >= entries.len() {
                    return Err(RuntimeError::IteratorExhausted);
                }

                let value = entries[*index].0.clone();

                *index += 1;

                Ok(value)
            }

            // ============================================================
            // STRING
            //
            // string -> iterator des caractères
            // ============================================================
            IteratorKind::String { string, index } => {
                let string = string.borrow();

                let Object::String(value) = &*string else {
                    return Err(RuntimeError::TypeError);
                };

                let character = value
                    .chars()
                    .nth(*index)
                    .ok_or(RuntimeError::IteratorExhausted)?;

                *index += 1;

                Ok(Value::new_string(character.to_string()))
            }

            // ============================================================
            // LAZY ADAPTERS
            //
            // Ces itérateurs nécessitent la VM pour exécuter les callbacks.
            // ============================================================
            IteratorKind::Map { .. }
            | IteratorKind::Filter { .. }
            | IteratorKind::Take { .. }
            | IteratorKind::Skip { .. } => Err(RuntimeError::TypeError),
        }
    }
}

// pub fn iterator_to_array(
//     value: &Value,
//     vm: &mut crate::vm::machine::VirtualMachine,
// ) -> Result<Value, RuntimeError> {
//     let iterator = value.to_iterator()?;

//     let mut values = Vec::new();

//     loop {
//         match vm.iterator_next_value(&iterator) {
//             Ok(value) => values.push(value),

//             Err(RuntimeError::IteratorExhausted) => {
//                 break;
//             }

//             Err(error) => {
//                 return Err(error);
//             }
//         }
//     }

//     Ok(Value::new_array(values))
// }
// ================================================================
//                     MATERIALISATION
// ================================================================

/// Matérialise un itérable simple en tableau.
///
/// Les itérateurs fonctionnels sont exécutés par `iterator.collect()`
/// au niveau VM.
pub fn drain_to_array(value: &Value) -> Result<Value, RuntimeError> {
    let iterator = value.to_iterator()?;

    let mut elements = Vec::new();

    while iterator.iterator_has_next()? {
        elements.push(iterator.iterator_next()?);
    }

    Ok(Value::new_array(elements))
}

// ================================================================
// ITERATOR
// ================================================================
//
// Deux concepts bien distincts, comme en Python :
//
// - `Value::Range { start, stop, step }` : léger (3 f64, aucune allocation
//   sur le tas), immuable, RÉUTILISABLE — reste hors du système Object/Gc
//   (voir value.rs et object.rs pour la justification).
//
// - `Object::Iterator(IteratorState)`, enveloppé dans un `Value::Object` :
//   le curseur À ÉTAT, à usage unique, qui avance à chaque appel. Créé
//   fraîchement à chaque fois qu'on demande "donne-moi un itérateur" via
//   `to_iterator()`.
//
// Le compilateur ne connaît jamais le type concret de l'itérable : il émet
// toujours la même séquence GetIterator (une fois) / IteratorNext (à
// chaque tour), quelle que soit la nature réelle de la valeur itérée.

use crate::error::runtime_error::RuntimeError;
use crate::runtime::gc_handle::Gc;
use crate::runtime::object::Object;
use crate::runtime::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum IteratorState {
    Range { current: f64, stop: f64, step: f64 },
    /// `Gc<Object>` pointe vers un `Object::Array` — même poignée que
    /// celle référencée par la `Value::Object` d'origine, donc les
    /// mutations du tableau pendant le parcours restent visibles.
    Array { array: Gc<Object>, index: usize },
}

impl IteratorState {
    /// Réinitialise l'état à une valeur neutre, sans référence externe.
    /// Utilisé par le GC pour casser un cycle : un itérateur sur un
    /// tableau retient une référence vers ce tableau, ce qui peut
    /// participer à un cycle si l'itérateur est lui-même stocké quelque
    /// part de durable (ex. poussé dans le tableau qu'il parcourt).
    pub(crate) fn reset_for_gc(&mut self) {
        *self = IteratorState::Range {
            current: 0.0,
            stop: 0.0,
            step: 1.0,
        };
    }
}

impl Value {
    // ============================================================
    //                      CONSTRUCTION
    // ============================================================

    /// Range léger et réutilisable — utilisé par native_range(). Aucune
    /// allocation sur le tas, quelle que soit l'amplitude de l'intervalle.
    pub fn new_range(start: f64, stop: f64, step: f64) -> Self {
        Value::Range { start, stop, step }
    }

    fn new_range_iterator(start: f64, stop: f64, step: f64) -> Value {
        let handle = Gc::new(Object::Iterator(IteratorState::Range {
            current: start,
            stop,
            step,
        }));

        crate::runtime::gc::register_object(&handle);

        Value::Object(handle)
    }

    fn new_array_iterator(array: Gc<Object>) -> Value {
        let handle = Gc::new(Object::Iterator(IteratorState::Array { array, index: 0 }));

        crate::runtime::gc::register_object(&handle);

        Value::Object(handle)
    }

    // ============================================================
    //                      PROTOCOLE D'ITÉRATION
    // ============================================================
    //
    // Deux méthodes séparées (plutôt qu'une seule combinée) pour matcher
    // les 3 opcodes de la VM : GetIterator (une fois, avant la boucle),
    // puis IteratorHasNext / IteratorNext (à chaque tour). has_next ne
    // modifie jamais l'état — seul next avance le curseur.

    /// Convertit une valeur en itérateur À ÉTAT, fraîchement créé.
    ///
    /// - Un `Range` produit un nouveau curseur à chaque appel : parcourir
    ///   le même `range(5)` deux fois donne deux fois la séquence complète.
    /// - Un `Array` produit un curseur qui garde la MÊME poignée `Gc` :
    ///   les mutations du tableau pendant le parcours restent visibles.
    /// - Un `Iterator` déjà existant est retourné tel quel (passthrough).
    pub fn to_iterator(&self) -> Result<Value, RuntimeError> {
        match self {
            Value::Range { start, stop, step } => {
                Ok(Value::new_range_iterator(*start, *stop, *step))
            }

            Value::Object(handle) => match &*handle.borrow() {
                Object::Array(_) => Ok(Value::new_array_iterator(handle.clone())),
                Object::Iterator(_) => Ok(self.clone()),
                _ => Err(RuntimeError::NotIterable),
            },

            _ => Err(RuntimeError::NotIterable),
        }
    }

    /// Vérifie s'il reste un élément, SANS avancer le curseur.
    pub fn iterator_has_next(&self) -> Result<bool, RuntimeError> {
        let Value::Object(handle) = self else {
            return Err(RuntimeError::TypeError);
        };

        let Object::Iterator(state) = &*handle.borrow() else {
            return Err(RuntimeError::TypeError);
        };

        let has_next = match state {
            IteratorState::Range { current, stop, step } => {
                if *step >= 0.0 {
                    current < stop
                } else {
                    current > stop
                }
            }

            IteratorState::Array { array, index } => {
                let Object::Array(elements) = &*array.borrow() else {
                    // Ne peut normalement pas arriver : un IteratorState::Array
                    // pointe toujours vers un Object::Array par construction.
                    return Err(RuntimeError::TypeError);
                };

                *index < elements.len()
            }
        };

        Ok(has_next)
    }

    /// Avance le curseur d'un cran et retourne l'élément. À n'appeler que
    /// si `iterator_has_next` a préalablement renvoyé `true` — garanti par
    /// le bytecode généré par `compile_for_in`, jamais par du code
    /// utilisateur directement. `IteratorExhausted` est un filet de
    /// sécurité, pas un cas normal.
    pub fn iterator_next(&self) -> Result<Value, RuntimeError> {
        let Value::Object(handle) = self else {
            return Err(RuntimeError::TypeError);
        };

        let mut borrowed = handle.borrow_mut();

        let Object::Iterator(state) = &mut *borrowed else {
            return Err(RuntimeError::TypeError);
        };

        match state {
            IteratorState::Range { current, stop, step } => {
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

            IteratorState::Array { array, index } => {
                let Object::Array(elements) = &*array.borrow() else {
                    return Err(RuntimeError::TypeError);
                };

                if *index >= elements.len() {
                    return Err(RuntimeError::IteratorExhausted);
                }

                let value = elements[*index].clone();

                *index += 1;

                Ok(value)
            }
        }
    }
}

/// Matérialise n'importe quel itérable en un tableau concret — équivalent
/// de `list(x)` en Python. Utile maintenant que range() ne construit plus
/// de tableau par défaut : `list(range(10))` force la matérialisation
/// quand on a réellement besoin d'un tableau indexable/mutable.
pub fn drain_to_array(value: &Value) -> Result<Value, RuntimeError> {
    let iterator = value.to_iterator()?;

    let mut elements = Vec::new();

    while iterator.iterator_has_next()? {
        elements.push(iterator.iterator_next()?);
    }

    Ok(Value::new_array(elements))
}
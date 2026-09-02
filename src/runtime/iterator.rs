// ================================================================
// ITERATOR
// ================================================================
//
// Deux concepts bien distincts, comme en Python :
//
// - `Value::Range { start, stop, step }` : léger (3 f64, aucune allocation
//   sur le tas), immuable, RÉUTILISABLE. `range(5)` peut être parcouru
//   plusieurs fois, exactement comme l'objet `range` de Python — chaque
//   `for x in r { ... }` crée un curseur frais sans affecter les autres.
//
// - `Value::Iterator(Rc<RefCell<IteratorState>>)` : le curseur À ÉTAT, à
//   usage unique, qui avance à chaque appel. Créé fraîchement à chaque
//   fois qu'on demande "donne-moi un itérateur" via `to_iterator()`.
//
// Le compilateur ne connaît jamais le type concret de l'itérable : il émet
// toujours la même séquence GetIterator (une fois) / IteratorNext (à
// chaque tour), quelle que soit la nature réelle de la valeur itérée.

use std::cell::RefCell;
use std::rc::Rc;

use crate::error::runtime_error::RuntimeError;
use crate::runtime::gc;
use crate::runtime::value::{ArrayRef, Value};

#[derive(Debug, Clone, PartialEq)]
pub enum IteratorState {
    Range { current: f64, stop: f64, step: f64 },
    Array { array: ArrayRef, index: usize },
}

impl IteratorState {
    /// Réinitialise l'état à une valeur neutre, sans référence externe.
    /// Utilisé par le GC pour casser un cycle : un itérateur sur un
    /// tableau retient une référence vers ce tableau (Rc), ce qui peut
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
        let handle = Rc::new(RefCell::new(IteratorState::Range {
            current: start,
            stop,
            step,
        }));

        gc::register_iterator(&handle);

        Value::Iterator(handle)
    }

    fn new_array_iterator(array: ArrayRef) -> Value {
        let handle = Rc::new(RefCell::new(IteratorState::Array { array, index: 0 }));

        gc::register_iterator(&handle);

        Value::Iterator(handle)
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
    /// - Un `Array` produit un curseur qui garde une référence partagée
    ///   (Rc::clone) : les mutations du tableau pendant le parcours restent
    ///   visibles, comme un Vec mutable itéré en Rust.
    /// - Un `Iterator` déjà existant est retourné tel quel (passthrough).
    pub fn to_iterator(&self) -> Result<Value, RuntimeError> {
        match self {
            Value::Range { start, stop, step } => {
                Ok(Value::new_range_iterator(*start, *stop, *step))
            }

            Value::Array(array) => Ok(Value::new_array_iterator(Rc::clone(array))),

            Value::Iterator(_) => Ok(self.clone()),

            _ => Err(RuntimeError::NotIterable),
        }
    }

    /// Vérifie s'il reste un élément, SANS avancer le curseur.
    pub fn iterator_has_next(&self) -> Result<bool, RuntimeError> {
        match self {
            Value::Iterator(state) => {
                let state = state.borrow();

                let has_next = match &*state {
                    IteratorState::Range { current, stop, step } => {
                        if *step >= 0.0 {
                            current < stop
                        } else {
                            current > stop
                        }
                    }

                    IteratorState::Array { array, index } => *index < array.borrow().len(),
                };

                Ok(has_next)
            }

            _ => Err(RuntimeError::TypeError),
        }
    }

    /// Avance le curseur d'un cran et retourne l'élément. À n'appeler que
    /// si `iterator_has_next` a préalablement renvoyé `true` — garanti par
    /// le bytecode généré par `compile_for_in`, jamais par du code
    /// utilisateur directement. `IteratorExhausted` est un filet de
    /// sécurité, pas un cas normal.
    pub fn iterator_next(&self) -> Result<Value, RuntimeError> {
        match self {
            Value::Iterator(state) => {
                let mut state = state.borrow_mut();

                match &mut *state {
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

                        Ok(Value::Number(value))
                    }

                    IteratorState::Array { array, index } => {
                        let array = array.borrow();

                        if *index >= array.len() {
                            return Err(RuntimeError::IteratorExhausted);
                        }

                        let value = array[*index].clone();

                        *index += 1;

                        Ok(value)
                    }
                }
            }

            _ => Err(RuntimeError::TypeError),
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
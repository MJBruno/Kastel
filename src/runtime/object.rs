use std::cell::RefCell;
use std::rc::Rc;

use crate::error::runtime_error::RuntimeError;
use crate::module::module::ModuleInstance;
use crate::runtime::closure::Closure;
use crate::runtime::function::Function;
use crate::runtime::gc;
use crate::runtime::value::Value;
use crate::vm::machine::ObjUpvalue;

#[allow(dead_code)]
impl Value {
    // ============================================================
    // ARRAY
    // ============================================================

    pub fn new_array(elements: Vec<Value>) -> Self {
        let handle = Rc::new(RefCell::new(elements));

        // Enregistrement auprès du collecteur de cycles (voir runtime::gc) :
        // chaque tableau vivant mais inatteignable depuis les racines de la
        // VM sera détecté et son cycle cassé lors d'un futur mark & sweep.
        gc::register_array(&handle);

        Value::Array(handle)
    }

    pub fn array_get(&self, index: usize) -> Result<Value, RuntimeError> {
        match self {
            Value::Array(array) => {
                let array = array.borrow();

                array
                    .get(index)
                    .cloned()
                    .ok_or(RuntimeError::IndexOutOfBounds)
            }

            _ => Err(RuntimeError::TypeError),
        }
    }

    pub fn array_set(&self, index: usize, value: Value) -> Result<(), RuntimeError> {
        match self {
            Value::Array(array) => {
                let mut array = array.borrow_mut();

                let slot = array.get_mut(index).ok_or(RuntimeError::IndexOutOfBounds)?;

                *slot = value;

                Ok(())
            }

            _ => Err(RuntimeError::TypeError),
        }
    }

    pub fn array_len(&self) -> Result<usize, RuntimeError> {
        match self {
            Value::Array(array) => Ok(array.borrow().len()),

            _ => Err(RuntimeError::TypeError),
        }
    }

    pub fn array_push(&self, value: Value) -> Result<usize, RuntimeError> {
        match self {
            Value::Array(array) => {
                let mut array = array.borrow_mut();

                array.push(value);

                Ok(array.len())
            }

            _ => Err(RuntimeError::TypeError),
        }
    }

    pub fn array_pop(&self) -> Result<Value, RuntimeError> {
        match self {
            Value::Array(array) => {
                let mut array = array.borrow_mut();

                // Convention JS-like : pop() sur un tableau vide renvoie nil
                // plutôt que de lever une erreur.
                Ok(array.pop().unwrap_or(Value::Nil))
            }

            _ => Err(RuntimeError::TypeError),
        }
    }

    pub fn array_insert(&self, index: usize, value: Value) -> Result<usize, RuntimeError> {
        match self {
            Value::Array(array) => {
                let mut array = array.borrow_mut();

                let length = array.len();

                if index > length {
                    return Err(RuntimeError::ArrayIndexOutOfBounds { index, length });
                }

                array.insert(index, value);

                Ok(array.len())
            }

            _ => Err(RuntimeError::TypeError),
        }
    }

    pub fn array_remove(&self, index: usize) -> Result<Value, RuntimeError> {
        match self {
            Value::Array(array) => {
                let mut array = array.borrow_mut();

                let length = array.len();

                if index >= length {
                    return Err(RuntimeError::ArrayIndexOutOfBounds { index, length });
                }

                Ok(array.remove(index))
            }

            _ => Err(RuntimeError::TypeError),
        }
    }

    pub fn array_clear(&self) -> Result<(), RuntimeError> {
        match self {
            Value::Array(array) => {
                array.borrow_mut().clear();

                Ok(())
            }

            _ => Err(RuntimeError::TypeError),
        }
    }

    pub fn array_contains(&self, value: &Value) -> Result<bool, RuntimeError> {
        match self {
            Value::Array(array) => {
                let array = array.borrow();

                Ok(array.iter().any(|element| element == value))
            }

            _ => Err(RuntimeError::TypeError),
        }
    }

    // ============================================================
    // MODULE
    // ============================================================

    pub fn new_module(module: Rc<ModuleInstance>) -> Self {
        Value::Module(module)
    }

    pub fn module_get(&self, name: &str) -> Result<Value, RuntimeError> {
        match self {
            Value::Module(module) => module.get_export(name).cloned().ok_or_else(|| {
                RuntimeError::ModuleError(format!(
                    "Module '{}' does not export '{}'",
                    module.name, name
                ))
            }),

            _ => Err(RuntimeError::TypeError),
        }
    }

    // ============================================================
    //                      OBJET
    // ============================================================

    pub fn new_object(fields: Vec<(String, Value)>) -> Self {
        let handle = Rc::new(RefCell::new(fields));

        gc::register_object(&handle);

        Value::Object(handle)
    }

    pub fn object_get(&self, name: &str) -> Result<Value, RuntimeError> {
        match self {
            Value::Object(fields) => fields
                .borrow()
                .iter()
                .find(|(key, _)| key == name)
                .map(|(_, value)| value.clone())
                .ok_or_else(|| RuntimeError::ObjectFieldNotFound(name.to_string())),

            _ => Err(RuntimeError::TypeError),
        }
    }

    /// Assigne un champ. Contrairement à `array_set` (qui exige un index
    /// existant), une clé absente est simplement ajoutée — comportement
    /// dynamique façon JS plutôt qu'une erreur "champ inconnu".
    pub fn object_set(&self, name: &str, value: Value) -> Result<(), RuntimeError> {
        match self {
            Value::Object(fields) => {
                let mut fields = fields.borrow_mut();

                match fields.iter_mut().find(|(key, _)| key == name) {
                    Some((_, slot)) => *slot = value,
                    None => fields.push((name.to_string(), value)),
                }

                Ok(())
            }

            _ => Err(RuntimeError::TypeError),
        }
    }

    // ============================================================
    //                      ACCÈS UNIFIÉ AUX PROPRIÉTÉS
    // ============================================================
    //
    // Utilisé par les opcodes GetProperty/SetProperty : `user.name` et
    // `module.export` partagent la même syntaxe (Expression::Member), donc
    // la VM n'a pas besoin de savoir à la compilation lequel des deux
    // c'est — elle demande juste "récupère/assigne la propriété `name`" et
    // laisse le type réel de la valeur décider du comportement.

    pub fn get_property(&self, name: &str) -> Result<Value, RuntimeError> {
        match self {
            Value::Module(_) => self.module_get(name),
            Value::Object(_) => self.object_get(name),
            _ => Err(RuntimeError::TypeError),
        }
    }

    pub fn set_property(&self, name: &str, value: Value) -> Result<(), RuntimeError> {
        match self {
            // Les modules restent en lecture seule : leurs exports sont
            // figés à la compilation du module importé.
            Value::Object(_) => self.object_set(name, value),
            _ => Err(RuntimeError::TypeError),
        }
    }
}

// ============================================================
//                      CLOSURES
// ============================================================

/// Construit une closure et l'enregistre immédiatement auprès du GC.
/// Point de passage unique pour toute création de closure dans la VM —
/// impossible d'en créer une sans qu'elle soit suivie par le collecteur
/// (contrairement à un `Rc::new(RefCell::new(Closure { ... }))` construit
/// à la main, où l'enregistrement pourrait être oublié).
pub fn new_closure(
    function: Rc<Function>,
    upvalues: Vec<Rc<RefCell<ObjUpvalue>>>,
) -> Rc<RefCell<Closure>> {
    let handle = Rc::new(RefCell::new(Closure { function, upvalues }));

    gc::register_closure(&handle);

    handle
}

// ============================================================
//                      UPVALUES
// ============================================================

/// Construit une upvalue ouverte (pointant sur un slot de la pile) et
/// l'enregistre immédiatement auprès du GC.
pub fn new_upvalue(slot: usize) -> Rc<RefCell<ObjUpvalue>> {
    let handle = Rc::new(RefCell::new(ObjUpvalue { slot, closed: None }));

    gc::register_upvalue(&handle);

    handle
}
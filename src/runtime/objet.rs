use std::cell::RefCell;
use std::rc::Rc;

use crate::error::runtime_error::RuntimeError;
use crate::module::module::ModuleInstance;
use crate::runtime::closure::Closure;
use crate::runtime::function::Function;
use crate::runtime::gc;
use crate::runtime::gc_handle::Gc;
use crate::runtime::object::Object;
use crate::runtime::value::Value;
use crate::vm::machine::ObjUpvalue;

/// Construit un `Value::Object` autour de n'importe quelle variante
/// `Object`, en l'enregistrant systématiquement auprès du GC. Unique
/// point de passage pour créer un `Gc<Object>` dans tout le projet —
/// impossible d'en créer un sans qu'il soit suivi par le collecteur.
fn new_object(object: Object) -> Value {
    let handle = Gc::new(object);

    gc::register_object(&handle);

    Value::Object(handle)
}

#[allow(dead_code)]
impl Value {
    // ============================================================
    // STRING
    // ============================================================

    /// Retourne le contenu si cette valeur est une chaîne, sinon `None`.
    /// Clone le contenu (plutôt que d'exposer directement le `Ref` du
    /// `RefCell`) pour rester simple à l'usage : les appelants (native.rs,
    /// machine.rs) veulent presque toujours soit une copie immédiate, soit
    /// juste tester le type.
    pub fn as_string_value(&self) -> Option<String> {
        match self {
            Value::Object(handle) => match &*handle.borrow() {
                Object::String(s) => Some(s.clone()),
                _ => None,
            },

            _ => None,
        }
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::Object(handle) if matches!(&*handle.borrow(), Object::String(_)))
    }

    // ============================================================
    // ARRAY
    // ============================================================

    pub fn array_get(&self, index: usize) -> Result<Value, RuntimeError> {
        self.with_array(|array| {
            array.get(index).cloned().ok_or(RuntimeError::IndexOutOfBounds)
        })
    }

    pub fn array_set(&self, index: usize, value: Value) -> Result<(), RuntimeError> {
        self.with_array_mut(|array| {
            let slot = array.get_mut(index).ok_or(RuntimeError::IndexOutOfBounds)?;
            *slot = value;
            Ok(())
        })
    }

    pub fn array_len(&self) -> Result<usize, RuntimeError> {
        self.with_array(|array| Ok(array.len()))
    }

    pub fn array_push(&self, value: Value) -> Result<usize, RuntimeError> {
        self.with_array_mut(|array| {
            array.push(value);
            Ok(array.len())
        })
    }

    pub fn array_pop(&self) -> Result<Value, RuntimeError> {
        self.with_array_mut(|array| {
            // Convention JS-like : pop() sur un tableau vide renvoie nil
            // plutôt que de lever une erreur.
            Ok(array.pop().unwrap_or(Value::Nil))
        })
    }

    pub fn array_insert(&self, index: usize, value: Value) -> Result<usize, RuntimeError> {
        self.with_array_mut(|array| {
            let length = array.len();

            if index > length {
                return Err(RuntimeError::ArrayIndexOutOfBounds { index, length });
            }

            array.insert(index, value);

            Ok(array.len())
        })
    }

    pub fn array_remove(&self, index: usize) -> Result<Value, RuntimeError> {
        self.with_array_mut(|array| {
            let length = array.len();

            if index >= length {
                return Err(RuntimeError::ArrayIndexOutOfBounds { index, length });
            }

            Ok(array.remove(index))
        })
    }

    pub fn array_clear(&self) -> Result<(), RuntimeError> {
        self.with_array_mut(|array| {
            array.clear();
            Ok(())
        })
    }

    pub fn array_contains(&self, value: &Value) -> Result<bool, RuntimeError> {
        self.with_array(|array| Ok(array.iter().any(|element| element == value)))
    }

    /// Accès en lecture au `Vec<Value>` sous-jacent si cette valeur est
    /// bien un tableau, sinon `TypeError`. Centralise le "déballage"
    /// `Value::Object -> Object::Array` commun à toutes les méthodes
    /// `array_*` ci-dessus.
    fn with_array<R>(&self, f: impl FnOnce(&Vec<Value>) -> Result<R, RuntimeError>) -> Result<R, RuntimeError> {
        match self {
            Value::Object(handle) => match &*handle.borrow() {
                Object::Array(array) => f(array),
                _ => Err(RuntimeError::TypeError),
            },

            _ => Err(RuntimeError::TypeError),
        }
    }

    fn with_array_mut<R>(
        &self,
        f: impl FnOnce(&mut Vec<Value>) -> Result<R, RuntimeError>,
    ) -> Result<R, RuntimeError> {
        match self {
            Value::Object(handle) => match &mut *handle.borrow_mut() {
                Object::Array(array) => f(array),
                _ => Err(RuntimeError::TypeError),
            },

            _ => Err(RuntimeError::TypeError),
        }
    }

    // ============================================================
    // MODULE
    // ============================================================

    pub fn new_module(module: Rc<ModuleInstance>) -> Self {
        new_object(Object::Module(module))
    }

    pub fn module_get(&self, name: &str) -> Result<Value, RuntimeError> {
        match self {
            Value::Object(handle) => match &*handle.borrow() {
                Object::Module(module) => module.get_export(name).cloned().ok_or_else(|| {
                    RuntimeError::ModuleError(format!(
                        "Module '{}' does not export '{}'",
                        module.name, name
                    ))
                }),

                _ => Err(RuntimeError::TypeError),
            },

            _ => Err(RuntimeError::TypeError),
        }
    }

    // ============================================================
    //                      OBJET (littéral { clé: valeur })
    //
    // Représenté en interne par `Object::Dict` (voir object.rs pour la
    // raison du renommage) — le nom "objet" reste celui utilisé côté
    // langage Kastel et dans l'API publique de ces méthodes.
    // ============================================================

    pub fn new_object(fields: Vec<(String, Value)>) -> Self {
        new_object(Object::Dict(fields))
    }

    pub fn object_get(&self, name: &str) -> Result<Value, RuntimeError> {
        match self {
            Value::Object(handle) => match &*handle.borrow() {
                Object::Dict(fields) => fields
                    .iter()
                    .find(|(key, _)| key == name)
                    .map(|(_, value)| value.clone())
                    .ok_or_else(|| RuntimeError::ObjectFieldNotFound(name.to_string())),

                _ => Err(RuntimeError::TypeError),
            },

            _ => Err(RuntimeError::TypeError),
        }
    }

    /// Assigne un champ. Contrairement à `array_set` (qui exige un index
    /// existant), une clé absente est simplement ajoutée — comportement
    /// dynamique façon JS plutôt qu'une erreur "champ inconnu".
    pub fn object_set(&self, name: &str, value: Value) -> Result<(), RuntimeError> {
        match self {
            Value::Object(handle) => match &mut *handle.borrow_mut() {
                Object::Dict(fields) => {
                    match fields.iter_mut().find(|(key, _)| key == name) {
                        Some((_, slot)) => *slot = value,
                        None => fields.push((name.to_string(), value)),
                    }

                    Ok(())
                }

                _ => Err(RuntimeError::TypeError),
            },

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
            Value::Object(handle) => match &*handle.borrow() {
                Object::Module(_) => self.module_get(name),
                Object::Dict(_) => self.object_get(name),
                _ => Err(RuntimeError::TypeError),
            },

            _ => Err(RuntimeError::TypeError),
        }
    }

    pub fn set_property(&self, name: &str, value: Value) -> Result<(), RuntimeError> {
        match self {
            // Les modules restent en lecture seule : leurs exports sont
            // figés à la compilation du module importé.
            Value::Object(handle) => {
                // ⚠️ L'emprunt de `matches!` doit être entièrement terminé
                // AVANT d'appeler `object_set` (qui fait son propre
                // `borrow_mut()`) : le tenir plus longtemps ferait
                // paniquer avec "already borrowed" à chaque affectation.
                let is_dict = matches!(&*handle.borrow(), Object::Dict(_));

                if is_dict {
                    self.object_set(name, value)
                } else {
                    Err(RuntimeError::TypeError)
                }
            }

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
/// (contrairement à un `Gc::new(...)` construit à la main ailleurs, où
/// l'enregistrement pourrait être oublié).
pub fn new_closure(function: Rc<Function>, upvalues: Vec<Rc<RefCell<ObjUpvalue>>>) -> Gc<Object> {
    let handle = Gc::new(Object::Closure(Closure { function, upvalues }));

    gc::register_object(&handle);

    handle
}

/// Construit une valeur `Function` (utilisée par le compilateur pour
/// peupler le pool de constantes — c'est CE constant qu'`OP_CLOSURE`
/// retrouve ensuite à l'exécution pour fabriquer la closure elle-même).
pub fn new_function(function: Rc<Function>) -> Value {
    new_object(Object::Function(function))
}

// ============================================================
//                      UPVALUES
// ============================================================

/// Construit une upvalue ouverte (pointant sur un slot de la pile) et
/// l'enregistre immédiatement auprès du GC. Les upvalues restent en
/// dehors du système `Object`/`Gc` (voir gc.rs) : ce sont des cellules
/// internes au mécanisme de capture des closures, pas des valeurs Kastel
/// de première classe.
pub fn new_upvalue(slot: usize) -> Rc<RefCell<ObjUpvalue>> {
    let handle = Rc::new(RefCell::new(ObjUpvalue { slot, closed: None }));

    gc::register_upvalue(&handle);

    handle
}
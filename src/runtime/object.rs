// ================================================================
// OBJECT
// ================================================================
//
// Tout ce qui vit derrière un `Value::Object(Gc<Object>)`. Une seule
// enum, un seul point d'entrée pour le GC : marquer un
// `Gc<Object>` revient à regarder quelle variante il contient et à
// parcourir ses propres références internes.
//
// Volontairement absents d'`Object` :
// - `NativeFunction` : pointeur de fonction `Copy`, pas de cycle GC.
// - `Range` : valeur légère, sans allocation.
// ================================================================

use std::cell::RefCell;
use std::rc::Rc;

use crate::module::module::ModuleInstance;
use crate::runtime::closure::Closure;
use crate::runtime::function::Function;
use crate::runtime::gc_handle::Gc;
use crate::runtime::iterator::IteratorState;
use crate::runtime::upvalue::ObjUpvalue;
use crate::runtime::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    String(String),

    Array(Vec<Value>),

    /// Dictionnaire dynamique Kastel.
    ///
    /// Les clés sont des `Value`, pas uniquement des chaînes.
    /// L'ordre d'insertion est conservé.
    Dict(Vec<(Value, Value)>),

    Function(Rc<Function>),

    Closure(Closure),

    Iterator(IteratorState),

    Module(Rc<ModuleInstance>),
}

impl Object {
    /// Construit une closure et l'enregistre immédiatement auprès du GC.
    pub fn new_closure(
        function: Rc<Function>,
        upvalues: Vec<Rc<RefCell<ObjUpvalue>>>,
    ) -> Gc<Object> {
        let handle = Gc::new(Object::Closure(Closure { function, upvalues }));
        crate::runtime::gc::register_object(&handle);

        handle
    }

    /// Casse un cycle de références pour les objets collectables.
    pub(crate) fn break_cycle(&mut self) {
        match self {
            Object::String(_) => {}

            Object::Array(elements) => {
                elements.clear();
            }

            Object::Dict(fields) => {
                fields.clear();
            }

            Object::Function(_) => {}

            Object::Closure(closure) => {
                closure.upvalues.clear();
            }

            Object::Iterator(state) => {
                state.reset_for_gc();
            }

            Object::Module(_) => {}
        }
    }
}

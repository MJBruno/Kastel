use std::cell::RefCell;
use std::collections::HashMap;
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

    /// Séquence ordonnée IMMUABLE, produite par un littéral `(a, b, c)`.
    /// Même représentation mémoire qu'Array (un `Vec<Value>` suivi par
    /// le GC), mais aucune méthode de mutation ne l'expose : voir
    /// `Value::new_tuple` / `Value::tuple_get` dans `runtime::value`.
    Tuple(Vec<Value>),

    Dict(Vec<(Value, Value)>),

    Function(Rc<Function>),

    Closure(Closure),

    BoundMethod {
        method: Option<Gc<Object>>,
        receiver: Value,
    },

    Iterator(IteratorState),

    Module(Rc<ModuleInstance>),

    Class {
        name: String,
        superclass: Option<Gc<Object>>,
        interfaces: Vec<Gc<Object>>,
        methods: HashMap<String, Value>,
    },

    Interface {
        name: String,
        bases: Vec<Gc<Object>>,
        methods: HashMap<String, usize>,
    },

    Instance {
        class: Option<Gc<Object>>,
        fields: HashMap<String, Value>,
    },
}

impl Object {
    pub fn new_closure(
        function: Rc<Function>,
        upvalues: Vec<Rc<RefCell<ObjUpvalue>>>,
        global_env: std::rc::Weak<RefCell<HashMap<String, Value>>>,
    ) -> Gc<Object> {
        let handle = Gc::new(Object::Closure(Closure {
            function,
            upvalues,
            global_env,
            owner_class: None,
        }));

        crate::runtime::gc::register_object(&handle);

        handle
    }

    pub(crate) fn break_cycle(&mut self) {
        match self {
            Object::String(_) => {}

            Object::Array(elements) => {
                elements.clear();
            }

            Object::Tuple(elements) => {
                elements.clear();
            }

            Object::Dict(fields) => {
                fields.clear();
            }

            Object::Function(_) => {}

            Object::Closure(closure) => {
                closure.upvalues.clear();
                closure.global_env = std::rc::Weak::new();
                closure.owner_class = None;
            }

            Object::BoundMethod { method, receiver } => {
                /*
                 * Unreachable BoundMethod :
                 * casser les références internes qui peuvent participer
                 * à un cycle.
                 */
                *receiver = Value::None;

                /*
                 * Il n'existe pas de valeur "vide" pour Gc<Object>.
                 * On ne peut donc pas remplacer `method`.
                 *
                 * Le cycle sera cassé par le nettoyage du propriétaire
                 * qui contenait normalement le BoundMethod.
                 */
                let _ = method;
            }

            Object::Iterator(state) => {
                state.reset_for_gc();
            }

            Object::Module(module) => {
                /*
                 * ModuleInstance est partagé par Rc et possède son propre
                 * graphe interne. On ne peut pas simplement remplacer le
                 * Rc ici sans modifier sa représentation.
                 *
                 * Le GC marque les exports accessibles. Pour un module
                 * totalement inaccessible, le Rc sera libéré quand le
                 * dernier propriétaire externe disparaîtra.
                 */
                let _ = module;
            }

            Object::Class {
                superclass,
                interfaces,
                methods,
                ..
            } => {
                /*
                 * Le nom n'introduit pas de cycle.
                 */
                *superclass = None;
                interfaces.clear();
                methods.clear();
            }

            Object::Interface { bases, methods, .. } => {
                bases.clear();
                methods.clear();
            }

            Object::Instance { class, fields } => {
                /*
                 * Une instance inaccessible ne doit plus retenir sa classe
                 * ni ses champs.
                 */
                fields.clear();

                /*
                 * `Gc<Object>` n'a pas de valeur nulle. La référence de
                 * classe reste donc temporairement présente jusqu'à la
                 * destruction complète de l'objet.
                 */
                let _ = class;
            }
        }
    }
}

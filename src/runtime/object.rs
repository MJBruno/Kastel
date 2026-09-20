use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
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

    /// Ensemble MUTABLE d'éléments UNIQUES (`Set(1, 2, 3)`, `{1, 2, 3}`).
    ///
    /// Même représentation qu'Array (`Vec<Value>` suivi par le GC), mais
    /// l'unicité est garantie par les seuls points d'entrée
    /// `Value::new_set` / `Value::set_add` (voir `runtime::value`). L'ordre
    /// d'insertion est conservé en interne, mais il ne fait PAS partie du
    /// contrat : un programme ne doit pas s'y fier.
    Set(Vec<Value>),

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
        /// Surcharges par arité : pour un nom donné, une closure par
        /// nombre d'arguments (hors `this`).
        methods: HashMap<String, Vec<Value>>,
        /// Noms des membres (champs et méthodes) déclarés `private` dans
        /// CETTE classe. L'accès n'est permis que depuis le corps de la
        /// classe (voir `VirtualMachine::ensure_member_access`).
        private_members: HashSet<String>,
    },

    Interface {
        name: String,
        bases: Vec<Gc<Object>>,
        /// Signatures exigées, identifiées par le couple `(nom, arité)` :
        /// une interface peut donc déclarer `area()` ET `area(unit)`.
        methods: HashSet<(String, usize)>,
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

            Object::Set(elements) => {
                elements.clear();
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

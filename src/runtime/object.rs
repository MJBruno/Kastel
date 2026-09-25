use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::module::module::ModuleInstance;
use crate::runtime::closure::Closure;
use crate::runtime::function::Function;
use crate::runtime::gc_handle::Gc;
use crate::runtime::hashed::{DictEntries, SetElements};
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

    Dict(DictEntries),

    /// Record : `{ name: "Bruno", age: 25 }`. Champs NOMMÉS (identifiants), en
    /// nombre FIXE, dans l'ordre de déclaration. Accès par `p.name` ; on peut
    /// modifier la valeur d'un champ existant mais pas en ajouter ni en
    /// retirer (pour cela : `Dict`). Comme Array et Dict, c'est un objet
    /// PARTAGÉ (`let b = a;` désigne le même record ; `a.copy()` le copie).
    Record(Vec<(String, Value)>),

    /// Ensemble MUTABLE d'éléments UNIQUES (`Set(1, 2, 3)`, `{1, 2, 3}`).
    ///
    /// Même représentation qu'Array (`Vec<Value>` suivi par le GC), mais
    /// l'unicité est garantie par les seuls points d'entrée
    /// `Value::new_set` / `Value::set_add` (voir `runtime::value`). L'ordre
    /// d'insertion est conservé en interne, mais il ne fait PAS partie du
    /// contrat : un programme ne doit pas s'y fier.
    Set(SetElements),

    Function(Rc<Function>),

    /// Fonction GLOBALE surchargée par arité : `func f(a) {}` + `func f(a, b) {}`.
    /// Les fermetures sont rangées dans l'ordre de déclaration ; un appel
    /// choisit celle dont le nombre de paramètres égale celui des arguments.
    /// Se passe comme une valeur (`let g = f;`) et s'exporte comme un seul nom.
    Overloads {
        name: String,
        functions: Vec<Value>,
    },

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
        /// Méthodes `static` : surcharges par arité, comme `methods`, mais
        /// SANS `this` (une closure de moins par arité). Appelées sur la
        /// classe elle-même (`NomClasse.methode(...)`), jamais héritées.
        static_methods: HashMap<String, Vec<Value>>,
        /// Champs `static` : une seule valeur par nom, portée par la classe
        /// (et non par chaque instance). Mutable via `NomClasse.champ = v`.
        /// Jamais hérité : une classe dérivée n'a pas accès aux statiques de
        /// sa base sous son propre nom.
        statics: HashMap<String, Value>,
        /// Noms des membres déclarés `private` dans CETTE classe.
        private_members: HashSet<String>,
        /// Noms des membres déclarés `protected` dans CETTE classe.
        protected_members: HashSet<String>,
    },

    Interface {
        name: String,
        bases: Vec<Gc<Object>>,
        /// Signatures exigées, identifiées par le couple `(nom, arité)` :
        /// une interface peut donc déclarer `area()` ET `area(unit)`.
        methods: HashSet<(String, usize)>,
    },

    /// Définition runtime d'un enum. Les variants sont des valeurs stables
    /// stockées ici et accessibles par `EnumName.Variant`.
    Enum {
        name: String,
        variants: HashMap<String, Value>,
    },

    /// Valeur singleton d'un variant d'enum. Les méthodes sont partagées par
    /// tous les variants via les mêmes handles de closure, sans créer de
    /// cycle GC enum -> variant -> enum.
    EnumVariant {
        enum_name: String,
        variant_name: String,
        methods: HashMap<String, Vec<Value>>,
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

            Object::Record(fields) => {
                fields.clear();
            }

            Object::Overloads { functions, .. } => {
                functions.clear();
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
                static_methods,
                statics,
                ..
            } => {
                /*
                 * Le nom n'introduit pas de cycle.
                 */
                *superclass = None;
                interfaces.clear();
                methods.clear();
                static_methods.clear();
                statics.clear();
            }

            Object::Interface { bases, methods, .. } => {
                bases.clear();
                methods.clear();
            }

            Object::Enum { variants, .. } => {
                variants.clear();
            }

            Object::EnumVariant { methods, .. } => {
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

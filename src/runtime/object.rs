use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::vm::machine::scheduler::TaskHandle;

use crate::module::module::ModuleInstance;
use crate::runtime::barrier::BarrierState;
use crate::runtime::channel::ChannelState;
use crate::runtime::closure::Closure;
use crate::runtime::condvar::CondvarState;
use crate::runtime::event::EventState;
use crate::runtime::function::Function;
use crate::runtime::gc_handle::Gc;
use crate::runtime::hashed::{DictEntries, SetElements};
use crate::runtime::iterator::IteratorState;
use crate::runtime::rwlock::RwLockState;
use crate::runtime::upvalue::ObjUpvalue;
use crate::runtime::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    String(String),

    Array(Vec<Value>),

    /// Valeur optionnelle : `Some(value)` ou `None`.
    /// `None` existe aussi comme valeur primitive Kastel ; la VM traite les
    /// deux représentations comme la même absence sémantique.
    Option(Option<Value>),

    /// Résultat : `Ok(value)` ou `Err(error)`.
    Result {
        ok: bool,
        value: Value,
    },

    /// Erreur runtime catchable par `catch(e: Err)`.
    /// `kind` identifie la famille d'erreur (`TypeError`, `DivisionByZero`, ...).
    /// `message` contient son message utilisateur.
    Error {
        kind: String,
        message: String,
    },

    /// Handle d'une tâche coopérative.
    Task(Rc<TaskHandle>),

    /// Canal coopératif, non borné ou borné selon `ChannelState::capacity`.
    Channel(Rc<RefCell<ChannelState>>),

    /// Mutex coopératif non réentrant.
    Mutex(Rc<RefCell<crate::runtime::mutex::MutexState>>),

    /// Sémaphore coopératif à compteur.
    Semaphore(Rc<RefCell<crate::runtime::semaphore::SemaphoreState>>),

    /// Compteur de coordination coopératif.
    WaitGroup(Rc<RefCell<crate::runtime::wait_group::WaitGroupState>>),

    /// Barrière coopérative réutilisable à nombre fixe de participants.
    Barrier(Rc<RefCell<BarrierState>>),

    /// RwLock coopératif équitable : lecteurs partagés, écrivain exclusif.
    RwLock(Rc<RefCell<RwLockState>>),

    /// Événement coopératif à état permanent.
    Event(Rc<RefCell<EventState>>),

    /// Variable de condition associée à un mutex coopératif.
    Condvar(Rc<RefCell<CondvarState>>),

    /// Ressource réseau opaque. L'état interne contient les sockets OS mais
    /// ne contient aucune `Value` : le GC n'a donc pas de graphe supplémentaire
    /// à parcourir pour ces objets.
    Network(Rc<RefCell<crate::runtime::net::NetworkState>>),

    /// Handle de fichier ouvert par `std.io` (`File`).
    File(Rc<RefCell<crate::runtime::file::FileState>>),

    /// Configuration d’ouverture de fichier (`OpenOptions`).
    OpenOptions(Rc<RefCell<crate::runtime::file::OpenOptionsState>>),

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
        /// Les membres statiques sont propres à la classe qui les déclare.
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

/// Destruction ITÉRATIVE : sans cela, libérer une chaîne profonde d'objets
/// (`a.add(b)`, `b.add(c)`, ... sur des dizaines de milliers de niveaux) fait
/// récurser `drop` sur la pile native jusqu'au débordement
/// (STATUS_STACK_OVERFLOW). Les enfants UNIQUEMENT détenus par cet objet sont
/// vidés avant d'être libérés, de sorte que chaque libération reste peu
/// profonde.
impl Drop for Object {
    fn drop(&mut self) {
        let mut pending: Vec<Gc<Object>> = Vec::new();

        self.detach_children(&mut pending);

        while let Some(child) = pending.pop() {
            // Un enfant encore partagé sera libéré par son dernier
            // propriétaire ; seul un enfant unique est vidé ici.
            if child.strong_count() == 1
                && let Ok(mut object) = child.try_borrow_mut()
            {
                object.detach_children(&mut pending);
            }

            // `child` est libéré ici, ses propres enfants déjà détachés.
        }
    }
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

    /// Détache de `self` ses enfants directs `Gc<Object>` (éléments de
    /// tableaux/tuples/sets/dicts/records, champs d'instance, ...) et les
    /// confie à `out`. À n'appeler que sur un objet en cours de destruction.
    ///
    /// Les poignées sont d'abord CLONÉES dans `out`, puis le conteneur est
    /// vidé : les enfants repassent à un seul propriétaire (`out`) sans être
    /// détruits ici.
    fn detach_children(&mut self, out: &mut Vec<Gc<Object>>) {
        fn collect(value: &Value, out: &mut Vec<Gc<Object>>) {
            if let Value::Object(handle) = value {
                out.push(handle.clone());
            }
        }

        match self {
            Object::Array(elements) | Object::Tuple(elements) => {
                for value in elements.iter() {
                    collect(value, out);
                }
                elements.clear();
            }

            Object::Option(value) => {
                if let Some(inner) = value.as_ref() {
                    collect(inner, out);
                }
                *value = None;
            }

            Object::Result { value, .. } => {
                collect(value, out);
                *value = Value::None;
            }

            Object::Record(fields) => {
                for (_, value) in fields.iter() {
                    collect(value, out);
                }
                fields.clear();
            }

            Object::Dict(entries) => {
                for (key, value) in entries.iter() {
                    collect(key, out);
                    collect(value, out);
                }
                entries.clear();
            }

            Object::Set(elements) => {
                for value in elements.iter() {
                    collect(value, out);
                }
                elements.clear();
            }

            Object::Overloads { functions, .. } => {
                for value in functions.iter() {
                    collect(value, out);
                }
                functions.clear();
            }

            Object::Instance { fields, .. } => {
                for value in fields.values() {
                    collect(value, out);
                }
                fields.clear();
            }

            Object::BoundMethod { receiver, .. } => {
                collect(receiver, out);
                *receiver = Value::None;
            }

            _ => {}
        }
    }

    pub(crate) fn break_cycle(&mut self) {
        match self {
            Object::String(_) => {}

            Object::Array(elements) => {
                elements.clear();
            }

            Object::Option(value) => {
                *value = None;
            }

            Object::Result { value, .. } => {
                *value = Value::None;
            }

            Object::Error { .. } => {}

            Object::Task(_) => {}

            Object::Channel(channel) => {
                channel.borrow_mut().queue.clear();
            }

            Object::Mutex(mutex) => {
                let mut mutex = mutex.borrow_mut();
                mutex.owner = None;
                mutex.waiters.clear();
            }

            Object::Semaphore(semaphore) => {
                semaphore.borrow_mut().waiters.clear();
            }

            Object::WaitGroup(wait_group) => {
                wait_group.borrow_mut().waiters.clear();
            }

            Object::Barrier(barrier) => {
                barrier.borrow_mut().waiters.clear();
            }

            Object::RwLock(rwlock) => {
                let mut rwlock = rwlock.borrow_mut();
                rwlock.writer = None;
                rwlock.readers.clear();
                rwlock.waiters.clear();
            }

            Object::Event(event) => {
                event.borrow_mut().waiters.clear();
            }

            Object::Condvar(condvar) => {
                condvar.borrow_mut().waiters.clear();
            }

            Object::Network(network) => {
                network.borrow_mut().close();
            }

            Object::File(file) => {
                file.borrow_mut().close();
            }

            Object::OpenOptions(_) => {}

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
                interfaces,
                methods,
                static_methods,
                statics,
                ..
            } => {
                /*
                 * Le nom n'introduit pas de cycle.
                 */
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

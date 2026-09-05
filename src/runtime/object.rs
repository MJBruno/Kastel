// ================================================================
// OBJECT
// ================================================================
//
// Tout ce qui vit derrière un `Value::Object(Gc<Object>)`. Une seule
// enum, un seul point d'entrée pour le GC (voir gc.rs) : marquer un
// `Gc<Object>` revient à regarder QUELLE variante il contient et à
// recurser dans ses propres `Value` internes, sans jamais avoir besoin
// de connaître le type concret en dehors de cette fonction de marquage.
//
// Volontairement absents d'`Object` (voir value.rs pour le pourquoi) :
// - `NativeFunction` : pointeur de fonction `Copy`, ne peut pas former de
//   cycle, n'a rien à faire dans un système de tracking mémoire.
// - `Range` : léger et volontairement réutilisable (aucune allocation),
//   c'est précisément ce qui rend `range()` paresseux — le faire passer
//   par `Gc` lui ferait perdre cette propriété.

use std::rc::Rc;

use crate::module::module::ModuleInstance;
use crate::runtime::closure::Closure;
use crate::runtime::function::Function;
use crate::runtime::iterator::IteratorState;
use crate::runtime::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    String(String),

    Array(Vec<Value>),

    /// Objet dynamique littéral `{ clé: valeur, ... }`. S'appelait
    /// `Value::Object` avant cette refonte — renommé `Dict` ici pour ne
    /// pas entrer en collision avec le type `Object` lui-même (qui
    /// désigne maintenant "tout ce que le GC suit", pas spécifiquement
    /// les littéraux `{ }`). Le nom visible depuis Kastel reste "objet".
    Dict(Vec<(String, Value)>),

    /// `Rc` interne conservé (plutôt que `Function` nu) : plusieurs
    /// closures créées à partir de la même fonction compilée (ex. une
    /// fonction imbriquée définie dans une boucle) doivent continuer à
    /// PARTAGER un seul exemplaire du bytecode compilé, pas en cloner un
    /// pour chacune. `Gc<Object>` donne l'unification de type ; ce `Rc`
    /// interne préserve le partage qui existait déjà avant la refonte.
    Function(Rc<Function>),

    Closure(Closure),

    Iterator(IteratorState),

    /// `Rc` interne conservé pour la même raison que `Function` : le
    /// `ModuleLoader` met en cache un `Rc<ModuleInstance>` par fichier, et
    /// plusieurs `import` du même module doivent partager cette instance
    /// plutôt que d'en recharger/recompiler une copie à chaque fois.
    Module(Rc<ModuleInstance>),
}

impl Object {
    /// Casse un cycle de références en vidant le contenu qui pourrait
    /// pointer vers d'autres objets. Utilisé par le sweep du GC (voir
    /// gc.rs) sur tout `Object` vivant mais inatteignable depuis les
    /// racines. `String`/`Function`/`Module` n'ont rien à casser : une
    /// chaîne ne référence aucune `Value`, une fonction compilée est
    /// immuable, et un module n'est jamais muté après son chargement par
    /// du code Kastel ordinaire — aucun des trois ne peut réalistement
    /// être le porteur d'un cycle.
    pub(crate) fn break_cycle(&mut self) {
        match self {
            Object::String(_) => {}
            Object::Array(elements) => elements.clear(),
            Object::Dict(fields) => fields.clear(),
            Object::Function(_) => {}
            Object::Closure(closure) => closure.upvalues.clear(),
            Object::Iterator(state) => state.reset_for_gc(),
            Object::Module(_) => {}
        }
    }
}
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};

use crate::runtime::closure::Closure;
use crate::runtime::value::Value;
use crate::vm::machine::{CallFrame, ObjUpvalue};

// ================================================================
// REGISTRE GLOBAL
//
// Kastel est mono-thread (tout le code utilise Rc/RefCell, jamais
// Arc/Mutex), donc un registre thread_local est sûr ici et évite de
// devoir faire transiter un `&mut Gc` à travers toutes les fonctions qui
// créent un tableau ou une closure (native.rs, value.rs, machine.rs...).
// ================================================================

thread_local! {
    static REGISTRY: RefCell<GcRegistry> = RefCell::new(GcRegistry::new());
}

struct GcRegistry {
    arrays: Vec<Weak<RefCell<Vec<Value>>>>,
    closures: Vec<Weak<RefCell<Closure>>>,
    upvalues: Vec<Weak<RefCell<ObjUpvalue>>>,
    allocations_since_collect: usize,
    threshold: usize,
}

impl GcRegistry {
    fn new() -> Self {
        Self {
            arrays: Vec::new(),
            closures: Vec::new(),
            upvalues: Vec::new(),
            allocations_since_collect: 0,
            // Seuil initial modeste ; doublé après chaque collecte (même
            // heuristique que `next_gc` dans clox) pour amortir le coût du
            // mark & sweep sur les programmes qui allouent beaucoup.
            threshold: 256,
        }
    }
}

/// Enregistre un tableau fraîchement alloué. À appeler à chaque création
/// d'un `Rc<RefCell<Vec<Value>>>` (voir `Value::new_array`).
pub fn register_array(handle: &Rc<RefCell<Vec<Value>>>) {
    REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        registry.arrays.push(Rc::downgrade(handle));
        registry.allocations_since_collect += 1;
    });
}

/// Enregistre une closure fraîchement allouée.
pub fn register_closure(handle: &Rc<RefCell<Closure>>) {
    REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        registry.closures.push(Rc::downgrade(handle));
        registry.allocations_since_collect += 1;
    });
}

/// Enregistre une upvalue fraîchement capturée.
pub fn register_upvalue(handle: &Rc<RefCell<ObjUpvalue>>) {
    REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        registry.upvalues.push(Rc::downgrade(handle));
        registry.allocations_since_collect += 1;
    });
}

/// Indique si le nombre d'allocations depuis la dernière collecte dépasse
/// le seuil courant. La VM appelle ceci entre deux instructions (jamais au
/// milieu d'un opcode) pour décider de déclencher `collect`.
pub fn should_collect() -> bool {
    REGISTRY.with(|registry| {
        let registry = registry.borrow();
        registry.allocations_since_collect >= registry.threshold
    })
}

/// Racines de la collecte : tout ce qui est directement accessible depuis
/// la VM et doit donc être considéré comme vivant, même si inatteignable
/// autrement (pile d'exécution, globales, closures des frames d'appel,
/// upvalues encore ouvertes).
pub struct GcRoots<'a> {
    pub stack: &'a [Value],
    pub globals: &'a HashMap<String, Value>,
    pub frames: &'a [CallFrame],
    pub open_upvalues: &'a [Rc<RefCell<ObjUpvalue>>],
}

/// Exécute un passage complet de collecte de cycles.
///
/// 1. MARK : parcourt récursivement toutes les racines et marque (par
///    identité de pointeur, via `Rc::as_ptr`) chaque tableau/closure/upvalue
///    atteignable.
/// 2. SWEEP : tout objet toujours enregistré et toujours vivant
///    (`Weak::upgrade` réussit) mais jamais marqué ne peut être maintenu en
///    vie que par un cycle de références internes — on vide son contenu
///    pour casser le cycle, ce qui laisse le comptage de références de Rc
///    terminer normalement le nettoyage (potentiellement en cascade sur
///    tout le reste du cycle).
///
/// Retourne le nombre de cycles cassés (utile pour du diagnostic).
pub fn collect(roots: GcRoots) -> usize {
    let mut marked_arrays: HashSet<usize> = HashSet::new();
    let mut marked_closures: HashSet<usize> = HashSet::new();
    let mut marked_upvalues: HashSet<usize> = HashSet::new();

    for value in roots.stack {
        mark_value(value, &mut marked_arrays, &mut marked_closures, &mut marked_upvalues);
    }

    for value in roots.globals.values() {
        mark_value(value, &mut marked_arrays, &mut marked_closures, &mut marked_upvalues);
    }

    for frame in roots.frames {
        mark_closure(&frame.closure, &mut marked_arrays, &mut marked_closures, &mut marked_upvalues);
    }

    for upvalue in roots.open_upvalues {
        mark_upvalue(upvalue, &mut marked_arrays, &mut marked_closures, &mut marked_upvalues);
    }

    REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        let mut broken = 0;

        registry.arrays.retain(|weak| match weak.upgrade() {
            Some(rc) => {
                let id = Rc::as_ptr(&rc) as usize;

                if !marked_arrays.contains(&id) {
                    // Vivant (refcount > 0) mais inatteignable depuis les
                    // racines : ne peut être maintenu en vie que par un
                    // cycle. On casse le cycle en vidant son contenu.
                    rc.borrow_mut().clear();
                    broken += 1;
                }

                true // toujours vivant (même vidé) : on le garde en registre
            }

            None => false, // complètement libéré : on peut l'oublier
        });

        registry.closures.retain(|weak| match weak.upgrade() {
            Some(rc) => {
                let id = Rc::as_ptr(&rc) as usize;

                if !marked_closures.contains(&id) {
                    rc.borrow_mut().upvalues.clear();
                    broken += 1;
                }

                true
            }

            None => false,
        });

        registry.upvalues.retain(|weak| match weak.upgrade() {
            Some(rc) => {
                let id = Rc::as_ptr(&rc) as usize;

                if !marked_upvalues.contains(&id) {
                    rc.borrow_mut().closed = None;
                    broken += 1;
                }

                true
            }

            None => false,
        });

        registry.allocations_since_collect = 0;
        registry.threshold = (registry.threshold * 2).max(256);

        broken
    })
}

fn mark_value(
    value: &Value,
    marked_arrays: &mut HashSet<usize>,
    marked_closures: &mut HashSet<usize>,
    marked_upvalues: &mut HashSet<usize>,
) {
    match value {
        Value::Array(array) => mark_array(array, marked_arrays, marked_closures, marked_upvalues),

        Value::Closure(closure) => {
            mark_closure(closure, marked_arrays, marked_closures, marked_upvalues)
        }

        _ => {}
    }
}

fn mark_array(
    array: &Rc<RefCell<Vec<Value>>>,
    marked_arrays: &mut HashSet<usize>,
    marked_closures: &mut HashSet<usize>,
    marked_upvalues: &mut HashSet<usize>,
) {
    let id = Rc::as_ptr(array) as usize;

    // `insert` retourne false si déjà présent : protège aussi contre une
    // boucle infinie si le tableau se contient (directement ou indirectement).
    if !marked_arrays.insert(id) {
        return;
    }

    for value in array.borrow().iter() {
        mark_value(value, marked_arrays, marked_closures, marked_upvalues);
    }
}

fn mark_closure(
    closure: &Rc<RefCell<Closure>>,
    marked_arrays: &mut HashSet<usize>,
    marked_closures: &mut HashSet<usize>,
    marked_upvalues: &mut HashSet<usize>,
) {
    let id = Rc::as_ptr(closure) as usize;

    if !marked_closures.insert(id) {
        return;
    }

    for upvalue in &closure.borrow().upvalues {
        mark_upvalue(upvalue, marked_arrays, marked_closures, marked_upvalues);
    }
}

fn mark_upvalue(
    upvalue: &Rc<RefCell<ObjUpvalue>>,
    marked_arrays: &mut HashSet<usize>,
    marked_closures: &mut HashSet<usize>,
    marked_upvalues: &mut HashSet<usize>,
) {
    let id = Rc::as_ptr(upvalue) as usize;

    if !marked_upvalues.insert(id) {
        return;
    }

    // Tant que l'upvalue est ouverte, la valeur vit sur la pile de la VM
    // (déjà couverte par le scan de `roots.stack`). Une fois fermée, elle
    // ne vit plus que dans `closed` : c'est là qu'un cycle peut se cacher.
    if let Some(value) = &upvalue.borrow().closed {
        mark_value(value, marked_arrays, marked_closures, marked_upvalues);
    }
}
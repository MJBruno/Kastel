use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};
use std::time::Instant;

use crate::runtime::gc_handle::Gc;
use crate::runtime::iterator::IteratorState;
use crate::runtime::object::Object;
use crate::runtime::value::Value;
use crate::vm::machine::{CallFrame, ObjUpvalue};

// ================================================================
// TRAÇAGE DU GC
//
// Feature Cargo `trace_gc`, symétrique à `debug_trace` (qui trace les
// instructions de la VM) : même mécanisme, même endroit (Cargo.toml),
// pour que les deux traçages restent cohérents et faciles à combiner :
//
//   cargo build --features trace_gc                    # GC seul
//   cargo build --features debug_trace                 # VM seule
//   cargo build --features "debug_trace,trace_gc"       # les deux
//
// Inspiré du DEBUG_LOG_GC de clox (Crafting Interpreters, ch. 26).
// ================================================================

fn trace_enabled() -> bool {
    cfg!(feature = "trace_gc")
}

// ================================================================
// REGISTRE GLOBAL
//
// UN SEUL registre pour tout ce qui est `Gc<Object>` — String, Array,
// Dict, Function, Closure, Iterator, Module confondus. C'est le bénéfice
// direct de l'unification `Value::Object(Gc<Object>)` : avant cette
// refonte, il fallait un `Vec<Weak<...>>` PAR TYPE (5 registres séparés,
// 5 fonctions de marquage, 5 blocs de sweep dupliqués). Maintenant, une
// seule identité de "ce que le GC suit" — `Gc<Object>` — et une seule
// fonction de marquage qui distribue en interne selon la variante
// rencontrée (voir `mark_object` plus bas).
//
// Les upvalues (`ObjUpvalue`, définies dans vm/machine.rs) restent
// suivies séparément : ce sont des cellules internes au mécanisme de
// capture des closures, pas des valeurs Kastel de première classe — elles
// ne font pas partie du diagramme Value/Object voulu ici.
//
// Kastel est mono-thread (tout le code utilise Rc/RefCell, jamais
// Arc/Mutex), donc un registre thread_local est sûr et évite de devoir
// faire transiter un `&mut Gc` à travers toutes les fonctions qui créent
// un objet (native.rs, value.rs, machine.rs...).
// ================================================================

thread_local! {
    static REGISTRY: RefCell<GcRegistry> = RefCell::new(GcRegistry::new());
}

struct GcRegistry {
    objects: Vec<Weak<RefCell<Object>>>,
    upvalues: Vec<Weak<RefCell<ObjUpvalue>>>,
    allocations_since_collect: usize,
    threshold: usize,
}

impl GcRegistry {
    fn new() -> Self {
        Self {
            objects: Vec::new(),
            upvalues: Vec::new(),
            allocations_since_collect: 0,
            // Seuil initial modeste ; recalculé après chaque collecte en
            // fonction de ce qui survit réellement (voir plus bas), même
            // heuristique que `next_gc` dans clox mais proportionnelle au
            // tas vivant plutôt qu'un doublement aveugle.
            threshold: 256,
        }
    }
}

/// Enregistre un objet fraîchement alloué. Unique point d'entrée pour
/// TOUT ce qui devient un `Gc<Object>` — appelé depuis `Gc::new` elle-même
/// n'est pas possible sans coupler `gc_handle.rs` au registre, donc c'est
/// aux constructeurs de haut niveau (`Value::new_string`, `new_array`,
/// `objet::new_closure`, ...) d'appeler ceci juste après `Gc::new(...)`.
pub fn register_object(handle: &Gc<Object>) {
    REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        registry.objects.push(handle.downgrade());
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

/// Identités déjà marquées pendant un passage de collecte. Deux
/// ensembles seulement désormais (objets unifiés + upvalues), contre 5
/// avant cette refonte.
#[derive(Default)]
struct MarkState {
    objects: HashSet<usize>,
    upvalues: HashSet<usize>,
}

/// Exécute un passage complet de collecte de cycles.
///
/// 1. MARK : parcourt récursivement toutes les racines et marque (par
///    identité de pointeur) chaque `Gc<Object>`/upvalue atteignable.
/// 2. SWEEP : tout objet toujours enregistré et toujours vivant
///    (`Weak::upgrade` réussit) mais jamais marqué ne peut être maintenu en
///    vie que par un cycle de références internes — `Object::break_cycle`
///    vide son contenu pour casser le cycle, ce qui laisse le comptage de
///    références de Rc terminer normalement le nettoyage (potentiellement
///    en cascade sur tout le reste du cycle).
///
/// Retourne le nombre de cycles cassés (utile pour du diagnostic).
pub fn collect(roots: GcRoots) -> usize {
    let started_at = Instant::now();

    if trace_enabled() {
        eprintln!("-- gc begin");
    }

    let mut state = MarkState::default();

    for value in roots.stack {
        mark_value(value, &mut state);
    }

    for value in roots.globals.values() {
        mark_value(value, &mut state);
    }

    for frame in roots.frames {
        mark_object(&frame.closure, &mut state);
    }

    for upvalue in roots.open_upvalues {
        mark_upvalue(upvalue, &mut state);
    }

    if trace_enabled() {
        eprintln!(
            "   mark: {} objets, {} upvalues atteignables",
            state.objects.len(),
            state.upvalues.len(),
        );
    }

    REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();

        let objects_before = registry.objects.len();
        let mut objects_broken = 0;

        registry.objects.retain(|weak| match weak.upgrade() {
            Some(rc) => {
                let id = Rc::as_ptr(&rc) as usize;

                if !state.objects.contains(&id) {
                    // Vivant (refcount > 0) mais inatteignable depuis les
                    // racines : ne peut être maintenu en vie que par un
                    // cycle. On casse le cycle en vidant son contenu.
                    rc.borrow_mut().break_cycle();
                    objects_broken += 1;
                }

                true // toujours vivant (même vidé) : on le garde en registre
            }

            None => false, // complètement libéré : on peut l'oublier
        });

        let upvalues_before = registry.upvalues.len();
        let mut upvalues_broken = 0;

        registry.upvalues.retain(|weak| match weak.upgrade() {
            Some(rc) => {
                let id = Rc::as_ptr(&rc) as usize;

                if !state.upvalues.contains(&id) {
                    rc.borrow_mut().closed = None;
                    upvalues_broken += 1;
                }

                true
            }

            None => false,
        });

        let broken = objects_broken + upvalues_broken;

        // Heuristique proportionnelle à la taille du tas VIVANT après la
        // collecte (mesurée juste après les .retain() ci-dessus) plutôt
        // qu'un doublement aveugle du seuil précédent : le rythme de
        // collecte s'adapte à ce qui survit réellement, comme dans
        // V8/CPython, pas au nombre brut d'allocations.
        let live_count = registry.objects.len() + registry.upvalues.len();

        registry.allocations_since_collect = 0;
        registry.threshold = (live_count * 2).max(256);

        if trace_enabled() {
            eprintln!(
                "   sweep: objets {objects_before} -> {} ({objects_broken} cycles cassés)",
                registry.objects.len()
            );
            eprintln!(
                "          upvalues {upvalues_before} -> {} ({upvalues_broken} cycles cassés)",
                registry.upvalues.len()
            );
            eprintln!(
                "-- gc end (total: {broken} cycles cassés, {:.3}ms, prochain seuil: {})",
                started_at.elapsed().as_secs_f64() * 1000.0,
                registry.threshold
            );
        }

        broken
    })
}

fn mark_value(value: &Value, state: &mut MarkState) {
    if let Value::Object(handle) = value {
        mark_object(handle, state);
    }
}

/// Marque un `Gc<Object>` et recurse dans son contenu selon sa variante
/// réelle. C'est le SEUL endroit de tout le GC qui a besoin de connaître
/// la forme interne d'`Object` — avant cette refonte, cette connaissance
/// était éclatée entre `mark_array`/`mark_closure`/`mark_object`/
/// `mark_iterator`, une fonction par type.
fn mark_object(handle: &Gc<Object>, state: &mut MarkState) {
    let id = handle.as_id();

    // Protège aussi contre une boucle infinie si l'objet se contient
    // (directement ou indirectement, ex. `arr.push(arr)`).
    if !state.objects.insert(id) {
        return;
    }

    match &*handle.borrow() {
        Object::String(_) => {}

        Object::Array(elements) => {
            for value in elements {
                mark_value(value, state);
            }
        }

        Object::Dict(fields) => {
            for (_, value) in fields {
                mark_value(value, state);
            }
        }

        // Une fonction compilée est immuable et ne référence aucune
        // Value de première classe directement (ses constantes vivent
        // dans son propre Chunk, pas dans le graphe d'objets du GC).
        Object::Function(_) => {}

        Object::Closure(closure) => {
            for upvalue in &closure.upvalues {
                mark_upvalue(upvalue, state);
            }
        }

        Object::Iterator(IteratorState::Range { .. }) => {}

        Object::Iterator(IteratorState::Array { array, .. }) => {
            mark_object(array, state);
        }

        Object::Module(module) => {
            for value in module.exports.values() {
                mark_value(value, state);
            }
        }
    }
}

fn mark_upvalue(upvalue: &Rc<RefCell<ObjUpvalue>>, state: &mut MarkState) {
    let id = Rc::as_ptr(upvalue) as usize;

    if !state.upvalues.insert(id) {
        return;
    }

    // Tant que l'upvalue est ouverte, la valeur vit sur la pile de la VM
    // (déjà couverte par le scan de `roots.stack`). Une fois fermée, elle
    // ne vit plus que dans `closed` : c'est là qu'un cycle peut se cacher.
    if let Some(value) = &upvalue.borrow().closed {
        mark_value(value, state);
    }
}
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};
use std::time::Instant;

use crate::runtime::closure::Closure;
use crate::runtime::iterator::IteratorState;
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
    objects: Vec<Weak<RefCell<Vec<(String, Value)>>>>,
    iterators: Vec<Weak<RefCell<IteratorState>>>,
    allocations_since_collect: usize,
    threshold: usize,
}

impl GcRegistry {
    fn new() -> Self {
        Self {
            arrays: Vec::new(),
            closures: Vec::new(),
            upvalues: Vec::new(),
            objects: Vec::new(),
            iterators: Vec::new(),
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

/// Enregistre un objet ({ clé: valeur, ... }) fraîchement alloué.
/// Un objet peut désormais participer à un cycle (ex. `a.self = a;`),
/// exactement comme un tableau — il doit donc être suivi de la même façon.
pub fn register_object(handle: &Rc<RefCell<Vec<(String, Value)>>>) {
    REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        registry.objects.push(Rc::downgrade(handle));
        registry.allocations_since_collect += 1;
    });
}

/// Enregistre un itérateur fraîchement créé (Range paresseux ou itérateur
/// de tableau). Un itérateur Array retient une référence vers son
/// tableau : rare mais possible qu'il participe à un cycle si l'itérateur
/// lui-même est stocké quelque part de durable — d'où le suivi, comme
/// pour les autres types du tas.
pub fn register_iterator(handle: &Rc<RefCell<IteratorState>>) {
    REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        registry.iterators.push(Rc::downgrade(handle));
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

/// Regroupe les ensembles d'identités déjà marquées pendant un passage de
/// collecte, par type d'objet. Une seule struct plutôt que 4 `HashSet`
/// threadés séparément à travers chaque fonction `mark_*` : plus simple à
/// lire, et plus facile à étendre si un futur type d'objet doit rejoindre
/// le GC (il suffit d'ajouter un champ ici plutôt que de retoucher toutes
/// les signatures existantes).
#[derive(Default)]
struct MarkState {
    arrays: HashSet<usize>,
    closures: HashSet<usize>,
    upvalues: HashSet<usize>,
    objects: HashSet<usize>,
    iterators: HashSet<usize>,
}

/// Exécute un passage complet de collecte de cycles.
///
/// 1. MARK : parcourt récursivement toutes les racines et marque (par
///    identité de pointeur, via `Rc::as_ptr`) chaque tableau/closure/
///    upvalue/objet atteignable.
/// 2. SWEEP : tout objet toujours enregistré et toujours vivant
///    (`Weak::upgrade` réussit) mais jamais marqué ne peut être maintenu en
///    vie que par un cycle de références internes — on vide son contenu
///    pour casser le cycle, ce qui laisse le comptage de références de Rc
///    terminer normalement le nettoyage (potentiellement en cascade sur
///    tout le reste du cycle).
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
        mark_closure(&frame.closure, &mut state);
    }

    for upvalue in roots.open_upvalues {
        mark_upvalue(upvalue, &mut state);
    }

    if trace_enabled() {
        eprintln!(
            "   mark: {} tableaux, {} closures, {} upvalues, {} objets atteignables",
            state.arrays.len(),
            state.closures.len(),
            state.upvalues.len(),
            state.objects.len(),
        );
    }

    REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        let mut broken = 0;

        let arrays_before = registry.arrays.len();
        let mut arrays_broken = 0;

        registry.arrays.retain(|weak| match weak.upgrade() {
            Some(rc) => {
                let id = Rc::as_ptr(&rc) as usize;

                if !state.arrays.contains(&id) {
                    // Vivant (refcount > 0) mais inatteignable depuis les
                    // racines : ne peut être maintenu en vie que par un
                    // cycle. On casse le cycle en vidant son contenu.
                    rc.borrow_mut().clear();
                    arrays_broken += 1;
                }

                true // toujours vivant (même vidé) : on le garde en registre
            }

            None => false, // complètement libéré : on peut l'oublier
        });

        broken += arrays_broken;

        let closures_before = registry.closures.len();
        let mut closures_broken = 0;

        registry.closures.retain(|weak| match weak.upgrade() {
            Some(rc) => {
                let id = Rc::as_ptr(&rc) as usize;

                if !state.closures.contains(&id) {
                    rc.borrow_mut().upvalues.clear();
                    closures_broken += 1;
                }

                true
            }

            None => false,
        });

        broken += closures_broken;

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

        broken += upvalues_broken;

        let objects_before = registry.objects.len();
        let mut objects_broken = 0;

        registry.objects.retain(|weak| match weak.upgrade() {
            Some(rc) => {
                let id = Rc::as_ptr(&rc) as usize;

                if !state.objects.contains(&id) {
                    rc.borrow_mut().clear();
                    objects_broken += 1;
                }

                true
            }

            None => false,
        });

        broken += objects_broken;

        let iterators_before = registry.iterators.len();
        let mut iterators_broken = 0;

        registry.iterators.retain(|weak| match weak.upgrade() {
            Some(rc) => {
                let id = Rc::as_ptr(&rc) as usize;

                if !state.iterators.contains(&id) {
                    rc.borrow_mut().reset_for_gc();
                    iterators_broken += 1;
                }

                true
            }

            None => false,
        });

        broken += iterators_broken;

        // Heuristique proportionnelle à la taille du tas VIVANT après la
        // collecte (mesurée juste après les .retain() ci-dessus, donc sans
        // les entrées totalement libérées) plutôt qu'un doublement aveugle
        // du seuil précédent :
        // - beaucoup de mémoire libérée -> tas vivant petit -> seuil bas
        //   -> prochaines collectes fréquentes mais très rapides (peu
        //   d'objets à parcourir).
        // - peu de mémoire libérée (gros jeu de données légitimement vivant)
        //   -> seuil élevé -> on évite de repasser inutilement souvent sur
        //   un tas qui ne contient presque jamais de cycles à casser.
        // C'est le même principe que la croissance du tas dans V8/CPython :
        // le rythme de collecte s'adapte à ce qui survit réellement, pas au
        // nombre brut d'allocations.
        let live_count = registry.arrays.len()
            + registry.closures.len()
            + registry.upvalues.len()
            + registry.objects.len()
            + registry.iterators.len();

        registry.allocations_since_collect = 0;
        registry.threshold = (live_count * 2).max(256);

        if trace_enabled() {
            eprintln!(
                "   sweep: tableaux {arrays_before} -> {} ({arrays_broken} cycles cassés)",
                registry.arrays.len()
            );
            eprintln!(
                "          closures {closures_before} -> {} ({closures_broken} cycles cassés)",
                registry.closures.len()
            );
            eprintln!(
                "          upvalues {upvalues_before} -> {} ({upvalues_broken} cycles cassés)",
                registry.upvalues.len()
            );
            eprintln!(
                "          objets   {objects_before} -> {} ({objects_broken} cycles cassés)",
                registry.objects.len()
            );
            eprintln!(
                "          itérateurs {iterators_before} -> {} ({iterators_broken} cycles cassés)",
                registry.iterators.len()
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
    match value {
        Value::Array(array) => mark_array(array, state),

        Value::Closure(closure) => mark_closure(closure, state),

        Value::Object(object) => mark_object(object, state),

        Value::Iterator(iterator) => mark_iterator(iterator, state),

        _ => {}
    }
}

fn mark_array(array: &Rc<RefCell<Vec<Value>>>, state: &mut MarkState) {
    let id = Rc::as_ptr(array) as usize;

    // `insert` retourne false si déjà présent : protège aussi contre une
    // boucle infinie si le tableau se contient (directement ou indirectement).
    if !state.arrays.insert(id) {
        return;
    }

    for value in array.borrow().iter() {
        mark_value(value, state);
    }
}

fn mark_closure(closure: &Rc<RefCell<Closure>>, state: &mut MarkState) {
    let id = Rc::as_ptr(closure) as usize;

    if !state.closures.insert(id) {
        return;
    }

    for upvalue in &closure.borrow().upvalues {
        mark_upvalue(upvalue, state);
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

fn mark_object(object: &Rc<RefCell<Vec<(String, Value)>>>, state: &mut MarkState) {
    let id = Rc::as_ptr(object) as usize;

    if !state.objects.insert(id) {
        return;
    }

    for (_, value) in object.borrow().iter() {
        mark_value(value, state);
    }
}

fn mark_iterator(iterator: &Rc<RefCell<IteratorState>>, state: &mut MarkState) {
    let id = Rc::as_ptr(iterator) as usize;

    if !state.iterators.insert(id) {
        return;
    }

    // Un itérateur Range ne référence aucune Value (juste 3 f64) : rien à
    // marquer. Un itérateur Array référence son tableau source : il faut
    // le marquer explicitement, sinon un tableau uniquement gardé en vie
    // par un itérateur stocké quelque part serait collecté à tort.
    if let IteratorState::Array { array, .. } = &*iterator.borrow() {
        mark_array(array, state);
    }
}
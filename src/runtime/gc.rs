use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};
use std::time::Instant;

use crate::module::module::ModuleInstance;
use crate::runtime::function::Function;
use crate::runtime::gc_handle::Gc;
use crate::runtime::object::Object;
use crate::runtime::upvalue::ObjUpvalue;
use crate::runtime::value::Value;
use crate::vm::machine::{CallFrame, PendingException};

fn trace_enabled() -> bool {
    cfg!(feature = "trace_gc")
}

thread_local! {
static REGISTRY: RefCell<GcRegistry> =
RefCell::new(GcRegistry::new());
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
            threshold: 256,
        }
    }
}

pub fn register_object(handle: &Gc<Object>) {
    REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();

        registry.objects.push(handle.downgrade());
        registry.allocations_since_collect += 1;
    });
}

pub fn register_upvalue(handle: &Rc<RefCell<ObjUpvalue>>) {
    REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();

        registry.upvalues.push(Rc::downgrade(handle));
        registry.allocations_since_collect += 1;
    });
}

pub fn should_collect() -> bool {
    REGISTRY.with(|registry| {
        let registry = registry.borrow();

        registry.allocations_since_collect >= registry.threshold
    })
}

/// Racines « épinglées » par une VM qui en lance une AUTRE (un `import`
/// exécute le module dans une VM imbriquée). Pendant que la VM imbriquée
/// tourne, ses collectes doivent aussi considérer comme vivantes les valeurs
/// de la VM appelante (pile, globales, frames...) : sans cela, le GC vidait
/// les structures du programme principal.
#[derive(Default)]
pub struct ExternalRoots {
    pub values: Vec<Value>,
    pub upvalues: Vec<Rc<RefCell<ObjUpvalue>>>,
}

thread_local! {
    static EXTERNAL_ROOTS: RefCell<Vec<ExternalRoots>> = RefCell::new(Vec::new());
}

/// Garde RAII : les racines épinglées le restent tant qu'elle existe.
/// Les gardes s'imbriquent (pile LIFO) : un module qui en importe un autre
/// conserve les racines de toute la chaîne d'appelants.
pub struct PinnedRoots(());

impl Drop for PinnedRoots {
    fn drop(&mut self) {
        EXTERNAL_ROOTS.with(|roots| {
            roots.borrow_mut().pop();
        });
    }
}

pub fn pin_roots(roots: ExternalRoots) -> PinnedRoots {
    EXTERNAL_ROOTS.with(|external| external.borrow_mut().push(roots));

    PinnedRoots(())
}

pub struct GcRoots<'a> {
    /// Valeurs tenues par du code natif pendant un rappel Kastel
    /// (`VirtualMachine::temp_roots`).
    pub temp: &'a [Value],

    pub stack: &'a [Value],
    pub globals: &'a HashMap<String, Value>,
    pub modules: &'a [Rc<ModuleInstance>],
    pub frames: &'a [CallFrame],
    pub open_upvalues: &'a [Rc<RefCell<ObjUpvalue>>],

    pub(crate) pending_exception: &'a Option<PendingException>,
}

#[derive(Default)]
struct MarkState {
    objects: HashSet<usize>,
    upvalues: HashSet<usize>,

    /// File des objets marqués dont les enfants restent à parcourir. Le
    /// marquage est ITÉRATIF : une structure imbriquée sur des dizaines de
    /// milliers de niveaux ne fait plus déborder la pile native.
    pending: Vec<Gc<Object>>,
}

pub fn collect(roots: GcRoots<'_>) -> usize {
    let started_at = Instant::now();

    if trace_enabled() {
        eprintln!("-- gc begin");
    }

    let mut state = MarkState::default();

    // Valeurs tenues par du code natif (rappels en cours)
    for value in roots.temp {
        mark_value(value, &mut state);
    }

    // Racines des VM appelantes (import en cours)
    EXTERNAL_ROOTS.with(|external| {
        for external_roots in external.borrow().iter() {
            for value in &external_roots.values {
                mark_value(value, &mut state);
            }

            for upvalue in &external_roots.upvalues {
                mark_upvalue(upvalue, &mut state);
            }
        }
    });

    // VM stack
    for value in roots.stack {
        mark_value(value, &mut state);
    }

    // Globals
    for value in roots.globals.values() {
        mark_value(value, &mut state);
    }

    // Environnements globaux des modules chargés.
    // Ils sont conservés par le cache du ModuleLoader, mais leurs valeurs
    // doivent tout de même être considérées comme des racines du GC.
    for module in roots.modules {
        for value in module.globals.borrow().values() {
            mark_value(value, &mut state);
        }

        for value in module.exports.values() {
            mark_value(value, &mut state);
        }
    }

    // Closures des frames actifs
    for frame in roots.frames {
        mark_object(&frame.closure, &mut state);
    }

    // Upvalues ouvertes
    for upvalue in roots.open_upvalues {
        mark_upvalue(upvalue, &mut state);
    }

    // Exception suspendue pendant finally
    if let Some(exception) = roots.pending_exception {
        mark_value(&exception.value, &mut state);
    }

    // Parcours (itératif) de tout ce qui est atteignable depuis les racines.
    drain_pending(&mut state);

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
        let mut objects_broken = 0usize;

        registry.objects.retain(|weak| match weak.upgrade() {
            Some(rc) => {
                let id = Rc::as_ptr(&rc) as usize;

                if !state.objects.contains(&id) {
                    rc.borrow_mut().break_cycle();
                    objects_broken += 1;
                }

                true
            }

            None => false,
        });

        let upvalues_before = registry.upvalues.len();
        let mut upvalues_broken = 0usize;

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

        // Adapter le prochain seuil à la taille du tas vivant.
        let live_count = registry.objects.len() + registry.upvalues.len();

        registry.allocations_since_collect = 0;
        registry.threshold = (live_count * 8).max(4096);

        if trace_enabled() {
            eprintln!(
                "   sweep: objets {} -> {} ({} cycles cassés)",
                objects_before,
                registry.objects.len(),
                objects_broken
            );

            eprintln!(
                "          upvalues {} -> {} ({} cycles cassés)",
                upvalues_before,
                registry.upvalues.len(),
                upvalues_broken
            );

            eprintln!(
                "-- gc end ({} cycles cassés, {:.3}ms, prochain seuil: {})",
                broken,
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

/// Marque `handle` et le met en file ; ses enfants sont parcourus par
/// `drain_pending` (pas de récursion).
fn mark_object(handle: &Gc<Object>, state: &mut MarkState) {
    let id = handle.as_id();

    if state.objects.insert(id) {
        state.pending.push(handle.clone());
    }
}

fn drain_pending(state: &mut MarkState) {
    while let Some(handle) = state.pending.pop() {
        trace_object(&handle, state);
    }
}

/// Marque les enfants directs de `handle` (ils sont mis en file, pas
/// visités ici).
fn trace_object(handle: &Gc<Object>, state: &mut MarkState) {
    match &*handle.borrow() {
        Object::String(_) => {}

        Object::Array(elements) => {
            for value in elements {
                mark_value(value, state);
            }
        }

        Object::Tuple(elements) => {
            for value in elements {
                mark_value(value, state);
            }
        }

        Object::Dict(fields) => {
            for (key, value) in fields {
                mark_value(key, state);
                mark_value(value, state);
            }
        }

        Object::Set(elements) => {
            for value in elements {
                mark_value(value, state);
            }
        }

        Object::Function(function) => {
            mark_function(function, state);
        }

        Object::Closure(closure) => {
            mark_function(&closure.function, state);

            for upvalue in &closure.upvalues {
                mark_upvalue(upvalue, state);
            }

            if let Some(owner_class) = &closure.owner_class {
                mark_object(owner_class, state);
            }
        }

        Object::BoundMethod { method, receiver } => {
            if let Some(method) = method {
                mark_object(method, state);
            }

            mark_value(receiver, state);
        }

        Object::Iterator(iterator) => {
            iterator.visit_values(|value| {
                mark_value(value, state);
            });
        }

        Object::Module(module) => {
            for value in module.globals.borrow().values() {
                mark_value(value, state);
            }

            for value in module.exports.values() {
                mark_value(value, state);
            }
        }

        Object::Class {
            superclass,
            interfaces,
            methods,
            ..
        } => {
            if let Some(superclass) = superclass {
                mark_object(superclass, state);
            }

            for interface in interfaces {
                mark_object(interface, state);
            }

            for overloads in methods.values() {
                for value in overloads {
                    mark_value(value, state);
                }
            }
        }

        Object::Interface { bases, methods, .. } => {
            for base in bases {
                mark_object(base, state);
            }

            let _ = methods;
        }

        Object::Instance { class, fields } => {
            if let Some(class) = class {
                mark_object(class, state);
            }

            for value in fields.values() {
                mark_value(value, state);
            }
        }
    }
}

fn mark_function(function: &Function, state: &mut MarkState) {
    for constant in &function.chunk.constants {
        mark_value(constant, state);
    }
}

fn mark_upvalue(upvalue: &Rc<RefCell<ObjUpvalue>>, state: &mut MarkState) {
    let id = Rc::as_ptr(upvalue) as usize;

    if !state.upvalues.insert(id) {
        return;
    }

    let upvalue_ref = upvalue.borrow();

    if let Some(value) = &upvalue_ref.closed {
        mark_value(value, state);
    }
}

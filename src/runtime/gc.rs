use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};
use std::time::Instant;

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

pub struct GcRoots<'a> {
    pub stack: &'a [Value],
    pub globals: &'a HashMap<String, Value>,
    pub frames: &'a [CallFrame],
    pub open_upvalues: &'a [Rc<RefCell<ObjUpvalue>>],

    pub(crate) pending_exception: &'a Option<PendingException>,
}

#[derive(Default)]
struct MarkState {
    objects: HashSet<usize>,
    upvalues: HashSet<usize>,
}

pub fn collect(roots: GcRoots<'_>) -> usize {
    let started_at = Instant::now();

    if trace_enabled() {
        eprintln!("-- gc begin");
    }

    let mut state = MarkState::default();

    // VM stack
    for value in roots.stack {
        mark_value(value, &mut state);
    }

    // Globals
    for value in roots.globals.values() {
        mark_value(value, &mut state);
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

<<<<<<< HEAD
<<<<<<< HEAD
        // Heuristique proportionnelle à la taille du tas VIVANT après la
        // collecte (mesurée juste après les .retain() ci-dessus) plutôt
        // qu'un doublement aveugle du seuil précédent : le rythme de
        // collecte s'adapte à ce qui survit réellement, comme dans
        // V8/CPython, pas au nombre brut d'allocations.
        let live_count = registry.objects.len() + registry.upvalues.len();

        registry.allocations_since_collect = 0;
        registry.threshold = (live_count * 8).max(4096);
=======
        registry.threshold =
            (live_count * 2).max(256);
>>>>>>> b172e95 (LSP)
=======
        let live_count = registry.objects.len() + registry.upvalues.len();

        registry.allocations_since_collect = 0;

        registry.threshold = (live_count * 2).max(256);
>>>>>>> 6a6d144 (Stabilisation de kastel)

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

fn mark_object(handle: &Gc<Object>, state: &mut MarkState) {
    let id = handle.as_id();

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
            for (key, value) in fields {
                mark_value(key, state);
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

            for value in methods.values() {
                mark_value(value, state);
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

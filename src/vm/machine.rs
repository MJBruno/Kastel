use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::rc::{Rc, Weak};

use crate::error::runtime_error::RuntimeError;
use crate::module::module::ModuleLoader;
use crate::runtime::channel::ChannelState;
use crate::runtime::function::Function;
use crate::runtime::gc_handle::Gc;
use crate::runtime::object::Object;
use crate::runtime::upvalue::ObjUpvalue;
use crate::runtime::value::Value;
use crate::stdlib::register_natives;

#[cfg(test)]
mod option_result_tests;
#[cfg(test)]
mod pattern_matching_tests;
#[cfg(test)]
mod exception_tests;
#[cfg(test)]
mod robustness_tests;
#[cfg(test)]
mod concurrency_tests;

pub mod arithmetic;
pub mod arrays;
pub mod bytecode;
pub mod calls;
pub mod classes;
pub mod closures;
pub mod control_flow;
pub mod debug;
pub mod dispatch;
pub mod exceptions;
pub mod execution;
pub mod gc;
pub mod iterators;
pub mod methods;
pub mod modules;
pub mod objects;
pub mod patterns;
pub mod profiling;
pub mod properties;
pub mod stack;
pub mod tuples;
pub mod variables;
pub mod scheduler;
#[derive(Clone)]
#[allow(dead_code)]
pub(crate) enum HotLoopCache {
    Integer {
        instruction_start: usize,
        local_index: usize,
        limit: i64,
        increment: i64,
    },
    Float {
        instruction_start: usize,
        local_index: usize,
        limit: f64,
        increment: f64,
    },
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct CallFrame {
    pub(crate) closure: Gc<Object>,
    pub(crate) chunk: Rc<crate::bytecode::chunk::Chunk>,
    pub(crate) ip: usize,
    pub(crate) slot_start: usize,
    pub(crate) local_count: usize,
    pub(crate) hot_loop_cache: Option<HotLoopCache>,
}

// ============================================================
// EXCEPTION HANDLER
// ============================================================

#[derive(Debug, Clone)]
pub(crate) struct ExceptionHandler {
    /*
     * Index du CallFrame dans lequel le handler a été créé.
     */
    pub(crate) frame_index: usize,

    /*
     * Adresse absolue dans le chunk du catch.
     */
    pub(crate) catch_ip: Option<usize>,

    /*
     * Adresse absolue dans le chunk du finally.
     */
    pub(crate) finally_ip: Option<usize>,

    /*
     * Hauteur de stack au moment du PushExceptionHandler.
     */
    pub(crate) stack_height: usize,

    /*
     * Type optionnel du catch (`Err` pour les erreurs runtime).
     * `None` = catch général.
     */
    pub(crate) catch_type: Option<String>,
}

// ============================================================
// PENDING EXCEPTION
// ============================================================

#[derive(Debug, Clone)]
pub(crate) struct PendingException {
    /*
     * Valeur lancée par throw.
     */
    pub(crate) value: Value,

    /*
     * true :
     * l'exception doit continuer sa propagation après finally.
     *
     * false :
     * finally termine simplement l'exécution normale.
     */
    pub(crate) rethrow: bool,
}

// ============================================================
// LIMITES
// ============================================================

/// Profondeur maximale de la pile d'appels Kastel (frames). Au-delà :
/// `RuntimeError::StackOverflow` (catchable par `try/catch`) au lieu d'une
/// consommation mémoire sans fin sur une récursion infinie.
pub(crate) const MAX_CALL_DEPTH: usize = 100_000;

/// Profondeur maximale de rappels NATIFS imbriqués (`map`, `filter`,
/// constructeurs, itérateurs... qui rappellent du code Kastel). Chaque niveau
/// consomme de la pile Rust : cette limite protège contre son débordement.
pub(crate) const MAX_NATIVE_DEPTH: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RunStatus {
    Completed,
    Yielded,
    Waiting,
}

// ============================================================
// VIRTUAL MACHINE
// ============================================================

#[allow(dead_code)]
pub struct VirtualMachine {
    pub stack: Vec<Value>,
    pub globals: Rc<RefCell<HashMap<String, Value>>>,

    pub(crate) frames: Vec<CallFrame>,

    /*
     * Pile des handlers actifs.
     *
     * Le dernier handler ajouté est le plus proche.
     */
    pub(crate) exception_handlers: Vec<ExceptionHandler>,

    /*
     * Exception temporairement suspendue pendant finally.
     */
    pub(crate) pending_exception: Option<PendingException>,

    pub(crate) open_upvalues: Vec<Rc<RefCell<ObjUpvalue>>>,

    /*
     * Racines TEMPORAIRES du GC : valeurs tenues uniquement par des
     * variables Rust pendant qu'un rappel Kastel s'exécute (résultats
     * intermédiaires de `map`, arguments d'un constructeur...). Sans elles,
     * le GC les croirait inaccessibles et VIDERAIT leur contenu.
     */
    pub(crate) temp_roots: Vec<Value>,

    /*
     * Profondeur courante de rappels natifs imbriqués (voir
     * `MAX_NATIVE_DEPTH`).
     */
    pub(crate) native_depth: usize,

    pub(crate) yield_requested: bool,
    pub(crate) waiting_requested: bool,
    pub(crate) task_id: Option<usize>,
    pub(crate) waiting_channel: Option<Value>,
    pub(crate) waiting_select_channels: Option<Vec<Value>>,
    pub(crate) waiting_timer: Option<Instant>,
    pub(crate) waiting_mutex: Option<Value>,
    pub(crate) waiting_semaphore: Option<Value>,
    pub(crate) waiting_wait_group: Option<Value>,
    pub(crate) waiting_error: Option<RuntimeError>,
    pub(crate) last_result: Option<Value>,
    pub(crate) scheduler: Weak<RefCell<scheduler::Scheduler>>,
    pub(crate) scheduler_owner: Option<Rc<RefCell<scheduler::Scheduler>>>,

    pub(crate) natives: HashMap<String, Value>,

    pub module_loader: ModuleLoader,

    pub module_path: Option<PathBuf>,

    pub current_line: usize,
    pub current_column: usize,

    #[cfg(feature = "profile")]
    pub(crate) profile_counts: [u64; 256],

    #[cfg(feature = "profile")]
    pub(crate) profile_read_bytes: u64,
}

impl VirtualMachine {
    pub fn new(function: Rc<Function>, module_path: Option<PathBuf>) -> Self {
        // La racine du projet est le répertoire de travail du
        // lancement ; en cas d'impossibilité, on utilise le dossier
        // du fichier d'entrée. Elle sert de repli pour la résolution des imports
        // "absolus" relatifs au projet, et est propagée telle quelle
        // à tous les modules chargés récursivement (le même
        // `ModuleLoader`, donc le même `ModuleResolver`, est cloné et
        // transmis à `execute_module` — voir `ModuleResolver`).
        let project_root = std::env::current_dir().unwrap_or_else(|_| {
            module_path
                .as_deref()
                .and_then(Path::parent)
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."))
        });

        Self::new_with_loader(function, module_path, ModuleLoader::new(project_root))
    }

    pub fn new_with_loader(
        function: Rc<Function>,
        module_path: Option<PathBuf>,
        module_loader: ModuleLoader,
    ) -> Self {
        let chunk = Rc::clone(&function.chunk);
        let local_count = function.local_count as usize;
        let globals = Rc::new(RefCell::new(HashMap::new()));
        let closure = Object::new_closure(function, Vec::new(), Rc::downgrade(&globals));

        let scheduler_owner = Rc::new(RefCell::new(scheduler::Scheduler::new()));
        let scheduler = Rc::downgrade(&scheduler_owner);

        let vm = Self {
            stack: vec![Value::None],

            globals: Rc::clone(&globals),

            frames: vec![CallFrame {
                closure,
                chunk,
                ip: 0,
                slot_start: 0,
                local_count,
                hot_loop_cache: None,
            }],

            exception_handlers: Vec::new(),
            pending_exception: None,
            open_upvalues: Vec::new(),
            temp_roots: Vec::new(),
            native_depth: 0,
            yield_requested: false,
            waiting_requested: false,
            task_id: None,
            waiting_channel: None,
            waiting_select_channels: None,
            waiting_timer: None,
            waiting_mutex: None,
            waiting_semaphore: None,
            waiting_wait_group: None,
            waiting_error: None,
            last_result: None,
            scheduler,
            scheduler_owner: Some(scheduler_owner),
            natives: HashMap::new(),

            module_loader,
            module_path,

            current_line: 0,
            current_column: 0,

            #[cfg(feature = "profile")]
            profile_counts: [0; 256],

            #[cfg(feature = "profile")]
            profile_read_bytes: 0,
        };

        register_natives(&mut vm.globals.borrow_mut());

        vm
    }

    fn new_with_loader_and_globals(
        function: Rc<Function>,
        module_path: Option<PathBuf>,
        module_loader: ModuleLoader,
        globals: Rc<RefCell<HashMap<String, Value>>>,
    ) -> Self {
        let chunk = Rc::clone(&function.chunk);
        let local_count = function.local_count as usize;
        let closure = Object::new_closure(function, Vec::new(), Rc::downgrade(&globals));

        let scheduler_owner = Rc::new(RefCell::new(scheduler::Scheduler::new()));
        let scheduler = Rc::downgrade(&scheduler_owner);

        let vm = Self {
            stack: vec![Value::None],
            globals,
            frames: vec![CallFrame {
                closure,
                chunk,
                ip: 0,
                slot_start: 0,
                local_count,
                hot_loop_cache: None,
            }],
            exception_handlers: Vec::new(),
            pending_exception: None,
            open_upvalues: Vec::new(),
            temp_roots: Vec::new(),
            native_depth: 0,
            yield_requested: false,
            waiting_requested: false,
            task_id: None,
            waiting_channel: None,
            waiting_select_channels: None,
            waiting_timer: None,
            waiting_mutex: None,
            waiting_semaphore: None,
            waiting_wait_group: None,
            waiting_error: None,
            last_result: None,
            scheduler,
            scheduler_owner: Some(scheduler_owner),
            natives: HashMap::new(),
            module_loader,
            module_path,
            current_line: 0,
            current_column: 0,
            #[cfg(feature = "profile")]
            profile_counts: [0; 256],
            #[cfg(feature = "profile")]
            profile_read_bytes: 0,
        };

        register_natives(&mut vm.globals.borrow_mut());

        vm
    }

    pub(crate) fn new_task(
        closure: Gc<Object>,
        arguments: Vec<Value>,
        globals: Rc<RefCell<HashMap<String, Value>>>,
        module_loader: ModuleLoader,
        module_path: Option<PathBuf>,
        scheduler: Weak<RefCell<scheduler::Scheduler>>,
        task_id: usize,
    ) -> Result<Self, RuntimeError> {
        let (arity, local_count, chunk, upvalue_count) = {
            let closure_ref = crate::vm::machine::bytecode::frame_closure(&closure);
            (
                closure_ref.function.arity,
                closure_ref.function.local_count as usize,
                Rc::clone(&closure_ref.function.chunk),
                closure_ref.upvalues.len(),
            )
        };

        if arguments.len() != arity {
            return Err(RuntimeError::WrongArgumentCount {
                expected: arity,
                found: arguments.len(),
            });
        }

        if upvalue_count != 0 {
            return Err(RuntimeError::TaskCaptureNotAllowed);
        }

        let mut stack = Vec::with_capacity(arguments.len() + 1);
        stack.push(Value::Object(closure.clone()));
        stack.extend(arguments);

        let vm = Self {
            stack,
            globals,
            frames: vec![CallFrame {
                closure,
                chunk,
                ip: 0,
                slot_start: 0,
                local_count,
                hot_loop_cache: None,
            }],
            exception_handlers: Vec::new(),
            pending_exception: None,
            open_upvalues: Vec::new(),
            temp_roots: Vec::new(),
            native_depth: 0,
            yield_requested: false,
            waiting_requested: false,
            task_id: Some(task_id),
            waiting_channel: None,
            waiting_select_channels: None,
            waiting_timer: None,
            waiting_mutex: None,
            waiting_semaphore: None,
            waiting_wait_group: None,
            waiting_error: None,
            last_result: None,
            scheduler,
            scheduler_owner: None,
            natives: HashMap::new(),
            module_loader,
            module_path,
            current_line: 0,
            current_column: 0,
            #[cfg(feature = "profile")]
            profile_counts: [0; 256],
            #[cfg(feature = "profile")]
            profile_read_bytes: 0,
        };

        // Les tâches partagent les globales de la VM racine : les natives
        // sont déjà enregistrées dans cette table et ne doivent pas être
        // réenregistrées à chaque spawn.
        Ok(vm)
    }

    pub(crate) fn wait_on_channel(
        &mut self,
        channel: Rc<RefCell<ChannelState>>,
        channel_value: Value,
    ) -> Result<(), RuntimeError> {
        let task_id = self
            .task_id
            .ok_or(RuntimeError::TaskNotFound)?;

        let scheduler = self
            .scheduler
            .upgrade()
            .ok_or(RuntimeError::TaskNotFound)?;

        scheduler::Scheduler::wait_on_channel(&scheduler, task_id, channel.clone())?;

        self.waiting_channel = Some(channel_value);
        self.waiting_requested = true;
        Ok(())
    }

    pub(crate) fn sleep_for(&mut self, duration: Duration) -> Result<(), RuntimeError> {
        let Some(task_id) = self.task_id else {
            // La VM racine n'est pas elle-même une tâche. Dans ce contexte,
            // on conserve une sémantique intuitive : `sleep()` suspend aussi
            // les tâches du scheduler pendant la durée demandée, sans créer
            // une fausse tâche bloquée.
            let deadline = Instant::now()
                .checked_add(duration)
                .ok_or(RuntimeError::InvalidFunction)?;

            let _pinned = self.pin_roots();
            let scheduler = self
                .scheduler
                .upgrade()
                .ok_or(RuntimeError::TaskNotFound)?;

            while Instant::now() < deadline {
                match scheduler::Scheduler::poll(&scheduler) {
                    Ok(true) => {}
                    Ok(false) | Err(RuntimeError::TaskDeadlock) => {
                        let remaining = deadline.saturating_duration_since(Instant::now());
                        if remaining.is_zero() {
                            break;
                        }
                        std::thread::sleep(remaining);
                        break;
                    }
                    Err(error) => return Err(error),
                }
            }
            return Ok(());
        };

        let scheduler = self
            .scheduler
            .upgrade()
            .ok_or(RuntimeError::TaskNotFound)?;

        let deadline = scheduler::Scheduler::sleep_task(&scheduler, task_id, duration)?;
        self.waiting_timer = Some(deadline);
        self.waiting_requested = true;
        Ok(())
    }

    pub(crate) fn wait_on_select(
        &mut self,
        channels: Vec<(Rc<RefCell<ChannelState>>, Value)>,
    ) -> Result<(), RuntimeError> {
        let task_id = self.task_id.ok_or(RuntimeError::TaskNotFound)?;

        let scheduler = self
            .scheduler
            .upgrade()
            .ok_or(RuntimeError::TaskNotFound)?;

        let values: Vec<Value> = channels.iter().map(|(_, value)| value.clone()).collect();
        scheduler::Scheduler::wait_on_select(&scheduler, task_id, &channels)?;

        self.waiting_select_channels = Some(values);
        self.waiting_channel = None;
        self.waiting_requested = true;
        Ok(())
    }

    pub(crate) fn resume_from_select(
        &mut self,
        index: usize,
        value: Value,
        closed: bool,
    ) -> Result<(), RuntimeError> {
        if self.waiting_select_channels.is_none() {
            return Err(RuntimeError::TaskNotFound);
        }

        self.waiting_select_channels = None;
        self.waiting_channel = None;
        self.waiting_error = None;
        self.waiting_requested = false;
        self.push(Value::new_tuple(vec![
            Value::Integer(index as i64),
            value,
            Value::Boolean(closed),
        ]));
        Ok(())
    }

    pub(crate) fn resume_from_timer(&mut self) -> Result<(), RuntimeError> {
        if self.waiting_timer.is_none() {
            return Err(RuntimeError::TaskNotFound);
        }

        let select_timeout = self.waiting_select_channels.is_some();
        self.waiting_timer = None;
        self.waiting_error = None;
        self.waiting_requested = false;

        if select_timeout {
            self.waiting_select_channels = None;
            self.push(Value::new_tuple(vec![
                Value::Integer(-1),
                Value::None,
                Value::Boolean(false),
            ]));
        }

        Ok(())
    }

    pub(crate) fn resume_from_mutex(&mut self) -> Result<(), RuntimeError> {
        if self.waiting_mutex.is_none() {
            return Err(RuntimeError::TaskNotFound);
        }

        self.waiting_mutex = None;
        self.waiting_error = None;
        self.waiting_requested = false;
        self.push(Value::None);
        Ok(())
    }

    pub(crate) fn resume_from_wait_group(&mut self) -> Result<(), RuntimeError> {
        if self.waiting_wait_group.is_none() {
            return Err(RuntimeError::TaskNotFound);
        }

        self.waiting_wait_group = None;
        self.waiting_error = None;
        self.waiting_requested = false;
        self.push(Value::None);
        Ok(())
    }

    pub(crate) fn resume_from_semaphore(&mut self) -> Result<(), RuntimeError> {
        if self.waiting_semaphore.is_none() {
            return Err(RuntimeError::TaskNotFound);
        }

        self.waiting_semaphore = None;
        self.waiting_error = None;
        self.waiting_requested = false;
        self.push(Value::None);
        Ok(())
    }

    pub(crate) fn resume_from_channel(&mut self, value: Value) -> Result<(), RuntimeError> {
        if self.waiting_channel.is_none() {
            return Err(RuntimeError::TaskNotFound);
        }

        self.waiting_channel = None;
        self.waiting_error = None;
        self.waiting_requested = false;
        self.push(value);
        Ok(())
    }

    pub(crate) fn resume_from_channel_error(
        &mut self,
        error: RuntimeError,
    ) -> Result<(), RuntimeError> {
        if self.waiting_channel.is_none() {
            return Err(RuntimeError::TaskNotFound);
        }

        self.waiting_channel = None;
        self.waiting_error = Some(error);
        self.waiting_requested = false;
        Ok(())
    }

    pub(crate) fn spawn_task(&mut self, arg_count: usize) -> Result<(), RuntimeError> {
        let required = arg_count
            .checked_add(1)
            .ok_or(RuntimeError::InvalidFunction)?;

        if self.stack.len() < required {
            return Err(RuntimeError::StackUnderflow);
        }

        let callee_index = self.stack.len() - required;
        let callee = self
            .stack
            .get(callee_index)
            .cloned()
            .ok_or(RuntimeError::StackUnderflow)?;

        let mut arguments = self
            .stack
            .get(callee_index + 1..)
            .ok_or(RuntimeError::StackUnderflow)?
            .to_vec();

        let closure = match callee {
            Value::Object(handle) => {
                let object = handle.borrow();

                match &*object {
                    Object::Closure(_) => handle.clone(),

                    Object::Overloads { functions, .. } => {
                        let selected = functions
                            .iter()
                            .find(|function| Self::function_arity(function) == Some(arg_count))
                            .cloned();

                        match selected {
                            Some(Value::Object(closure))
                                if matches!(&*closure.borrow(), Object::Closure(_)) => closure,

                            _ => {
                                let expected = functions
                                    .iter()
                                    .filter_map(Self::function_arity)
                                    .min()
                                    .unwrap_or(0);

                                return Err(RuntimeError::WrongArgumentCount {
                                    expected,
                                    found: arg_count,
                                });
                            }
                        }
                    }

                    Object::BoundMethod {
                        method: Some(method),
                        receiver,
                    } => {
                        arguments.insert(0, receiver.clone());
                        method.clone()
                    }

                    _ => return Err(RuntimeError::NotCallable),
                }
            }

            _ => return Err(RuntimeError::NotCallable),
        };

        let scheduler = self
            .scheduler
            .upgrade()
            .ok_or(RuntimeError::InvalidFunction)?;

        let handle = scheduler::Scheduler::spawn(
            &scheduler,
            closure,
            arguments,
            Rc::clone(&self.globals),
            self.module_loader.clone(),
            self.module_path.clone(),
        )?;

        self.stack.truncate(callee_index);
        let task_object = Gc::new(Object::Task(handle));
        crate::runtime::gc::register_object(&task_object);
        self.push(Value::Object(task_object));
        Ok(())
    }

    /// Libère la pile, les frames et les racines d'une tâche TERMINÉE. Sans
    /// cela, chaque tâche finie gardait sa pile entière tant que le scheduler
    /// vivait (fuite proportionnelle au nombre de `spawn`).
    pub(crate) fn release_task_resources(&mut self) {
        // Fermer d'abord les upvalues encore ouvertes : un objet partagé qui
        // survit à la tâche ne doit pas pointer vers une pile effacée. En cas
        // d'incohérence (échec en plein déroulement), on ne touche à rien.
        if self.close_upvalues(0).is_err() {
            return;
        }

        self.stack.clear();
        self.frames.clear();
        self.exception_handlers.clear();
        self.pending_exception = None;
        self.temp_roots.clear();
        self.waiting_channel = None;
        self.waiting_select_channels = None;
        self.waiting_timer = None;
        self.waiting_mutex = None;
        self.waiting_semaphore = None;
        self.waiting_wait_group = None;
        self.last_result = None;
        self.open_upvalues.clear();
    }

    pub(crate) fn last_result_value(&self) -> Value {
        self.last_result.clone().unwrap_or(Value::None)
    }

    pub fn execute_repl(&mut self, function: Rc<Function>) -> Result<Option<Value>, RuntimeError> {
        // Fermer les upvalues de l'ancien environnement avant de supprimer la stack.
        self.close_upvalues(0)?;

        let chunk = Rc::clone(&function.chunk);
        let local_count = function.local_count as usize;
        let closure = Object::new_closure(function, Vec::new(), Rc::downgrade(&self.globals));

        self.stack.clear();
        self.stack.push(Value::None);

        self.frames.clear();
        self.frames.push(CallFrame {
            closure,
            chunk,
            ip: 0,
            slot_start: 0,
            local_count,
            hot_loop_cache: None,
        });

        self.exception_handlers.clear();
        self.pending_exception = None;
        self.temp_roots.clear();
        self.native_depth = 0;
        self.yield_requested = false;
        self.waiting_requested = false;
        self.task_id = None;
        self.waiting_channel = None;
        self.waiting_select_channels = None;
        self.waiting_timer = None;
        self.waiting_mutex = None;
        self.waiting_semaphore = None;
        self.waiting_wait_group = None;
        self.last_result = None;

        // Les upvalues ont maintenant été fermées correctement.
        self.open_upvalues.clear();

        self.current_line = 0;
        self.current_column = 0;

        self.run()?;

        if self.stack.len() > 1 {
            Ok(self.stack.last().cloned())
        } else {
            Ok(None)
        }
    }
    pub fn execute_module(
        function: Rc<Function>,
        exports: &[String],
        module_path: PathBuf,
        module_loader: ModuleLoader,
        globals: Rc<RefCell<HashMap<String, Value>>>,
    ) -> Result<HashMap<String, Value>, RuntimeError> {
        let mut vm =
            Self::new_with_loader_and_globals(function, Some(module_path), module_loader, globals);

        if let Err(error) = vm.run() {
            return Err(RuntimeError::WithLocation {
                line: vm.current_line,
                column: vm.current_column,
                source: Box::new(error),
            });
        }

        let mut values = HashMap::with_capacity(exports.len());

        let globals = vm.globals.borrow();

        for name in exports {
            let value = globals.get(name).cloned().ok_or_else(|| {
                RuntimeError::ModuleError(format!("Export '{}' was not initialized", name))
            })?;

            values.insert(name.clone(), value);
        }

        Ok(values)
    }
    // ============================================================
    // UPVALUES
    // ============================================================

    pub(crate) fn capture_upvalue(
        &mut self,
        slot: usize,
    ) -> Result<Rc<RefCell<ObjUpvalue>>, RuntimeError> {
        let (slot_start, local_count) = {
            let frame = self.current_frame()?;
            let closure = crate::vm::machine::bytecode::frame_closure(&frame.closure);

            (frame.slot_start, closure.function.local_count as usize)
        };

        if slot >= local_count {
            return Err(RuntimeError::InvalidFunction);
        }

        let absolute_slot = slot_start
            .checked_add(1)
            .and_then(|value| value.checked_add(slot))
            .ok_or(RuntimeError::InvalidFunction)?;

        if absolute_slot >= self.stack.len() {
            return Err(RuntimeError::InvalidFunction);
        }

        if let Some(existing) = self
            .open_upvalues
            .iter()
            .find(|upvalue| upvalue.borrow().slot == absolute_slot)
        {
            return Ok(Rc::clone(existing));
        }

        let upvalue = Rc::new(RefCell::new(ObjUpvalue::new(absolute_slot)));

        crate::runtime::gc::register_upvalue(&upvalue);

        self.open_upvalues.push(Rc::clone(&upvalue));

        Ok(upvalue)
    }

    pub(crate) fn close_upvalues(&mut self, last: usize) -> Result<(), RuntimeError> {
        // Valider toutes les positions avant de modifier la liste.
        for upvalue in &self.open_upvalues {
            let upvalue_ref = upvalue.borrow();

            if upvalue_ref.slot >= last && upvalue_ref.closed.is_none()
                && upvalue_ref.slot >= self.stack.len() {
                    return Err(RuntimeError::InvalidFunction);
                }
        }

        // Fermer les upvalues qui appartiennent au frame supprimé.
        for upvalue in &self.open_upvalues {
            let slot = upvalue.borrow().slot;

            if slot >= last {
                let value = self
                    .stack
                    .get(slot)
                    .cloned()
                    .ok_or(RuntimeError::InvalidFunction)?;

                upvalue.borrow_mut().closed = Some(value);
            }
        }

        // Ne conserver ouvertes que les upvalues appartenant aux
        // frames encore actifs.
        self.open_upvalues
            .retain(|upvalue| upvalue.borrow().slot < last);

        Ok(())
    }


    pub(crate) fn register_exception_handler(
        &mut self,
        catch_ip: Option<usize>,
        finally_ip: Option<usize>,
        catch_type: Option<String>,
    ) -> Result<(), RuntimeError> {
        let frame_index = self
            .frames
            .len()
            .checked_sub(1)
            .ok_or(RuntimeError::InvalidFunction)?;

        self.exception_handlers.push(ExceptionHandler {
            frame_index,
            catch_ip,
            finally_ip,
            stack_height: self.stack.len(),
            catch_type,
        });

        Ok(())
    }

    pub(crate) fn unregister_exception_handler(
        &mut self,
    ) -> Result<ExceptionHandler, RuntimeError> {
        let frame_index = self
            .frames
            .len()
            .checked_sub(1)
            .ok_or(RuntimeError::InvalidFunction)?;

        let index = self
            .exception_handlers
            .iter()
            .rposition(|handler| handler.frame_index == frame_index)
            .ok_or(RuntimeError::InvalidFunction)?;

        Ok(self.exception_handlers.remove(index))
    }

    pub(crate) fn remove_handlers_for_frame(&mut self, frame_index: usize) {
        self.exception_handlers
            .retain(|handler| handler.frame_index != frame_index);
    }

    pub(crate) fn prune_exception_handlers(&mut self) {
        let frame_count = self.frames.len();

        self.exception_handlers
            .retain(|handler| handler.frame_index < frame_count);
    }

    // pub(crate) fn nearest_exception_handler(
    //     &self,
    // ) -> Option<&ExceptionHandler> {
    //     self.exception_handlers.last()
    // }

    // pub(crate) fn take_nearest_exception_handler(
    //     &mut self,
    // ) -> Option<ExceptionHandler> {
    //     self.exception_handlers.pop()
    // }

    pub(crate) fn restore_exception_stack(
        &mut self,
        stack_height: usize,
    ) -> Result<(), RuntimeError> {
        if stack_height > self.stack.len() {
            return Err(RuntimeError::InvalidFunction);
        }

        self.stack.truncate(stack_height);

        Ok(())
    }


    // ============================================================
    // FRAME CLEANUP
    // ============================================================

    pub(crate) fn remove_current_frame_handlers(&mut self) {
        if let Some(frame_index) = self.frames.len().checked_sub(1) {
            self.remove_handlers_for_frame(frame_index);
        }
    }

    pub(crate) fn close_current_frame_for_exception(&mut self) -> Result<(), RuntimeError> {
        let frame = match self.frames.last() {
            Some(frame) => frame.clone(),
            None => return Ok(()),
        };

        /*
         * Fermer les upvalues avant de supprimer les slots
         * appartenant au frame.
         */
        self.close_upvalues(frame.slot_start)?;

        /*
         * Le frame ne peut plus conserver de handler actif.
         */
        self.remove_current_frame_handlers();

        /*
         * Supprimer les valeurs appartenant au frame.
         */
        if self.stack.len() > frame.slot_start {
            self.stack.truncate(frame.slot_start);
        }

        /*
         * Retirer le frame.
         */
        self.frames.pop();

        /*
         * Nettoyer les handlers devenus invalides.
         */
        self.prune_exception_handlers();

        Ok(())
    }
}

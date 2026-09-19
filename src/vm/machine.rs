use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::error::runtime_error::RuntimeError;
use crate::module::module::ModuleLoader;
use crate::runtime::function::Function;
use crate::runtime::gc_handle::Gc;
use crate::runtime::object::Object;
use crate::runtime::upvalue::ObjUpvalue;
use crate::runtime::value::Value;
use crate::stdlib::register_natives;

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
pub mod profiling;
pub mod properties;
pub mod stack;
pub mod tuples;
pub mod variables;
pub mod patterns;
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
        let chunk = Rc::new(function.chunk.clone());
        let local_count = function.local_count as usize;
        let globals = Rc::new(RefCell::new(HashMap::new()));
        let closure = Object::new_closure(function, Vec::new(), Rc::downgrade(&globals));

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
        let chunk = Rc::new(function.chunk.clone());
        let local_count = function.local_count as usize;
        let closure = Object::new_closure(
            function,
            Vec::new(),
            Rc::downgrade(&globals),
        );

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

    pub fn execute_repl(&mut self, function: Rc<Function>) -> Result<Option<Value>, RuntimeError> {
        // Fermer les upvalues de l'ancien environnement avant de supprimer la stack.
        self.close_upvalues(0)?;

        let chunk = Rc::new(function.chunk.clone());
        let local_count = function.local_count as usize;
        let closure = Object::new_closure(
            function,
            Vec::new(),
            Rc::downgrade(&self.globals),
        );

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
        let mut vm = Self::new_with_loader_and_globals(
            function,
            Some(module_path),
            module_loader,
            globals,
        );

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

            if upvalue_ref.slot >= last && upvalue_ref.closed.is_none() {
                if upvalue_ref.slot >= self.stack.len() {
                    return Err(RuntimeError::InvalidFunction);
                }
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
    // ============================================================
    // EXCEPTION HANDLERS
    // ============================================================

    // pub(crate) fn current_frame_handler_count(
    //     &self,
    // ) -> usize {
    //     let frame_index =
    //         self.frames.len().saturating_sub(1);

    //     self.exception_handlers
    //         .iter()
    //         .filter(|handler| {
    //             handler.frame_index == frame_index
    //         })
    //         .count()
    // }

    pub(crate) fn register_exception_handler(
        &mut self,
        catch_ip: Option<usize>,
        finally_ip: Option<usize>,
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
    // PENDING EXCEPTION
    // ============================================================

    // pub(crate) fn set_pending_exception(
    //     &mut self,
    //     value: Value,
    // ) {
    //     self.pending_exception = Some(
    //         PendingException {
    //             value,
    //             rethrow: true,
    //         },
    //     );
    // }

    // pub(crate) fn take_pending_exception(
    //     &mut self,
    // ) -> Option<PendingException> {
    //     self.pending_exception.take()
    // }

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

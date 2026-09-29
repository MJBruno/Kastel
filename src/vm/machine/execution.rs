use super::VirtualMachine;

use crate::bytecode::chunk::OpCode;
use crate::error::runtime_error::RuntimeError;
use crate::runtime::gc;
use crate::runtime::object::Object;
use crate::runtime::value::Value;

impl VirtualMachine {
    pub fn run(&mut self) -> Result<(), RuntimeError> {
        loop {
            match self.run_internal(true, None)? {
                super::RunStatus::Completed => {
                    self.drain_tasks();
                    return Ok(());
                }
                super::RunStatus::Yielded => continue,
                super::RunStatus::Waiting => {
                    return Err(RuntimeError::TaskDeadlock);
                }
            }
        }
    }

    /// Fin de programme : les tâches lancées mais jamais jointes s'exécutent
    /// avant de rendre la main (voir `Scheduler::drain`). Seule la VM qui
    /// POSSÈDE le scheduler le fait ; une tâche n'en possède pas.
    fn drain_tasks(&mut self) {
        let Some(owner) = self.scheduler_owner.clone() else {
            return;
        };

        // Les valeurs de cette VM (résultat REPL, pile) doivent survivre aux
        // collectes déclenchées par les tâches.
        let _pinned = self.pin_roots();

        super::scheduler::Scheduler::drain(&owner);
    }

    #[allow(dead_code)]
    pub(crate) fn run_without_gc(&mut self) -> Result<(), RuntimeError> {
        loop {
            match self.run_internal(false, None)? {
                super::RunStatus::Completed => return Ok(()),
                super::RunStatus::Yielded => continue,
                super::RunStatus::Waiting => {
                    return Err(RuntimeError::TaskDeadlock);
                }
            }
        }
    }

    pub(crate) fn run_quantum(
        &mut self,
        instruction_budget: usize,
    ) -> Result<super::RunStatus, RuntimeError> {
        self.run_internal(true, Some(instruction_budget.max(1)))
    }

    fn run_internal(
        &mut self,
        allow_gc: bool,
        instruction_budget: Option<usize>,
    ) -> Result<super::RunStatus, RuntimeError> {
        let mut gc_check_counter = 0usize;
        let mut instructions = 0usize;

        loop {
            if let Some(error) = self.waiting_error.take() {
                if !self.propagate_runtime_error(error.clone())? {
                    self.print_profile();
                    return Err(error);
                }
            }

            if let Some(budget) = instruction_budget
                && instructions >= budget {
                    return Ok(super::RunStatus::Yielded);
                }
            instructions += 1;
            if cfg!(feature = "debug_trace") {
                self.debug_machine()?;
            }

            // Vérifier le GC périodiquement plutôt qu'à chaque instruction.
            if allow_gc {
                gc_check_counter += 1;

                if gc_check_counter >= 256 {
                    gc_check_counter = 0;

                    if gc::should_collect() {
                        self.collect_garbage();
                    }
                }
            }

            // Mettre à jour la position source avant l'exécution
            // de l'instruction courante.
            let (line, column) = self.current_position()?;

            self.current_line = line;
            self.current_column = column;

            let instruction = self.read_byte()?;

            #[cfg(feature = "profile")]
            self.profile_instruction(instruction);

            let result = match instruction {
                // ========================================================
                // HOT DISPATCH
                // ========================================================
                x if x == OpCode::Loop as u8 => self.loop_back().map(|_| false),

                x if x == OpCode::AddLocalConst as u8 => self.add_local_const().map(|_| false),

                x if x == OpCode::LessLocalConst as u8 => self.less_local_const().map(|_| false),

                x if x == OpCode::JumpIfFalsePop as u8 => self.jump_if_false_pop().map(|_| false),

                x if x == OpCode::LessLocalConstJump as u8 => self.less_local_const_jump().map(|_| false),

                x if x == OpCode::LoopLessAddLocalConst as u8 => self.loop_less_add_local_const().map(|_| false),

                x if x == OpCode::AddLocalLocal as u8 => self.add_local_local().map(|_| false),

                // ========================================================
                // GENERAL DISPATCH
                // ========================================================
                _ => self.dispatch(instruction),
            };

            match result {
                Ok(true) => {
                    self.print_profile();
                    return Ok(super::RunStatus::Completed);
                }

                Ok(false) => {}

                Err(error) => {
                    let (line, column) = self.current_position()?;

                    self.current_line = line;
                    self.current_column = column;

                    if !self.propagate_runtime_error(error.clone())? {
                        self.print_profile();
                        return Err(error);
                    }
                }
            }

            if self.waiting_requested {
                self.waiting_requested = false;
                return Ok(super::RunStatus::Waiting);
            }

            if self.yield_requested {
                self.yield_requested = false;
                return Ok(super::RunStatus::Yielded);
            }
        }
    }

    pub(crate) fn propagate_runtime_error(
        &mut self,
        error: RuntimeError,
    ) -> Result<bool, RuntimeError> {
        self.propagate_runtime_error_until(error, 0)
    }

    pub(crate) fn is_cancellation_error(error: &RuntimeError) -> bool {
        match error {
            RuntimeError::TaskCancelled => true,
            RuntimeError::WithLocation { source, .. } => Self::is_cancellation_error(source),
            _ => false,
        }
    }

    pub(crate) fn propagate_runtime_error_until(
        &mut self,
        error: RuntimeError,
        min_frame_len: usize,
    ) -> Result<bool, RuntimeError> {
        // Tâche en cours d'annulation : l'annulation n'est PAS interceptable
        // par `catch`, mais les `finally` doivent s'exécuter (libération de
        // verrous, `wait_group.done()`, fermeture de canaux...).
        if self.cancelling && Self::is_cancellation_error(&error) {
            return self.propagate_cancellation_until(min_frame_len);
        }

        match error {
            RuntimeError::Thrown(value) => self.propagate_thrown_until(value, min_frame_len),

            error => {
                let value = self.runtime_error_value(&error)?;

                if self.propagate_thrown_until(value, min_frame_len)? {
                    Ok(true)
                } else {
                    Err(error)
                }
            }
        }
    }

    fn runtime_error_value(&self, error: &RuntimeError) -> Result<Value, RuntimeError> {
        match error {
            RuntimeError::TypeError
            | RuntimeError::DivisionByZero
            | RuntimeError::WrongArgumentCount { .. }
            | RuntimeError::NotCallable
            | RuntimeError::NativeError
            | RuntimeError::OptionUnwrap { .. }
            | RuntimeError::ResultUnwrap { .. }
            | RuntimeError::IndexOutOfBounds
            | RuntimeError::InterfaceMethodMissing { .. }
            | RuntimeError::InterfaceMethodArityMismatch { .. }
            | RuntimeError::DuplicateMethod { .. }
            | RuntimeError::AmbiguousMethod { .. }
            | RuntimeError::PrivateMemberAccess { .. }
            | RuntimeError::ProtectedMemberAccess { .. }
            | RuntimeError::StackOverflow { .. }
            | RuntimeError::IntegerOverflow { .. }
            | RuntimeError::CyclicStructure
            | RuntimeError::ArrayIndexNotInteger
            | RuntimeError::ArrayIndexOutOfBounds { .. }
            | RuntimeError::NotIndexable
            | RuntimeError::NotObject
            | RuntimeError::ModuleError(_)
            | RuntimeError::FormatError(_)
            | RuntimeError::ObjectFieldNotFound { .. }
            | RuntimeError::NotIterable
            | RuntimeError::IteratorExhausted
            | RuntimeError::InvalidShiftAmount
            | RuntimeError::NumericTypeError { .. }
            | RuntimeError::TryOperandType { .. } => {
                Ok(Value::new_error(error.kind_name(), error.to_string()))
            }

            RuntimeError::TaskCaptureNotAllowed
            | RuntimeError::TaskNotFound
            | RuntimeError::YieldOutsideTask
            | RuntimeError::TaskDeadlock
            | RuntimeError::TaskCancelled
            | RuntimeError::TaskNestingTooDeep
            | RuntimeError::ChannelClosed
            | RuntimeError::ChannelNonPositive
            | RuntimeError::MutexDeadlock
            | RuntimeError::MutexNotOwner
            | RuntimeError::SemaphoreNonPositive
            | RuntimeError::SemaphoreNotOwner
            | RuntimeError::WaitGroupUnderflow
            | RuntimeError::WaitGroupNegativeCount
            | RuntimeError::BarrierNonPositive
            | RuntimeError::BarrierBroken
            | RuntimeError::RwLockDeadlock
            | RuntimeError::RwLockNotOwner
            | RuntimeError::CondvarDeadlock
            | RuntimeError::CondvarNotOwner => {
                Ok(Value::new_error(error.kind_name(), error.to_string()))
            }

            RuntimeError::Thrown(value) => Ok(value.clone()),

            RuntimeError::WithLocation { source, .. } => self.runtime_error_value(source),

            RuntimeError::StackUnderflow
            | RuntimeError::InvalidOpcode(_)
            | RuntimeError::InvalidFunction
            | RuntimeError::ImmutableValue(_) => Err(error.clone()),
        }
    }

    fn exception_matches_catch_type(catch_type: Option<&str>, value: &Value) -> bool {
        let Some(expected) = catch_type else {
            return true;
        };

        match value {
            Value::Object(handle) => match &*handle.borrow() {
                Object::Error { kind, .. } => {
                    expected.eq_ignore_ascii_case("Err")
                        || expected.eq_ignore_ascii_case(kind)
                }
                Object::Instance { class, .. } => class.as_ref().is_some_and(|class| {
                    match &*class.borrow() {
                        Object::Class { name, .. } => name.eq_ignore_ascii_case(expected),
                        _ => false,
                    }
                }),
                _ => value.type_name().eq_ignore_ascii_case(expected),
            },
            _ => value.type_name().eq_ignore_ascii_case(expected),
        }
    }

    pub(crate) fn propagate_thrown_until(
        &mut self,
        value: Value,
        min_frame_len: usize,
    ) -> Result<bool, RuntimeError> {
        loop {
            if self.frames.len() <= min_frame_len {
                return Ok(false);
            }

            let current_frame_index = self.frames.len() - 1;

            let Some(handler_index) = self.exception_handlers.iter().rposition(|handler| {
                handler.frame_index >= min_frame_len
                    && handler.frame_index <= current_frame_index
            }) else {
                self.close_current_frame_for_exception()?;
                continue;
            };

            let handler_frame = self.exception_handlers[handler_index].frame_index;

            while self.frames.len() > handler_frame + 1 {
                self.close_current_frame_for_exception()?;
            }

            if self.frames.len() <= min_frame_len {
                return Ok(false);
            }

            let catch_ip = self.exception_handlers[handler_index].catch_ip;
            let finally_ip = self.exception_handlers[handler_index].finally_ip;
            let stack_height = self.exception_handlers[handler_index].stack_height;

            // `catch_ip == None` : le handler garde seulement son `finally`
            // (l'exception a été levée DANS le `catch`, ou il n'y a pas de
            // `catch`). Il ne peut alors plus rien intercepter.
            let can_catch = catch_ip.is_some()
                && Self::exception_matches_catch_type(
                    self.exception_handlers[handler_index].catch_type.as_deref(),
                    &value,
                );

            self.restore_exception_stack(stack_height)?;

            if can_catch && let Some(catch_ip) = catch_ip {
                if finally_ip.is_some() {
                    // Le handler reste actif pendant le corps du `catch` pour
                    // que son `finally` s'exécute si le `catch` lève à son tour.
                    self.exception_handlers[handler_index].catch_ip = None;
                } else {
                    // Sans `finally`, plus rien à protéger : ne pas laisser un
                    // handler périmé intercepter les exceptions SUIVANTES.
                    self.exception_handlers.remove(handler_index);
                }

                self.push(value);
                self.current_frame_mut()?.ip = catch_ip;

                return Ok(true);
            }

            // Ce handler ne peut pas intercepter : il ne doit plus bloquer la
            // propagation. Son `finally`, s'il existe, s'exécute d'abord.
            self.exception_handlers.remove(handler_index);

            if let Some(finally_ip) = finally_ip {
                self.push(value);
                self.push(Value::Boolean(true));
                self.current_frame_mut()?.ip = finally_ip;

                return Ok(true);
            }

            // Pas de `finally` : chercher le handler suivant, dans ce frame ou
            // dans un frame appelant (le frame n'est fermé que s'il n'en reste
            // aucun).
        }
    }

    /// Propagation d'une annulation : seuls les handlers avec `finally` sont
    /// entrés, les `catch` sont ignorés.
    fn propagate_cancellation_until(
        &mut self,
        min_frame_len: usize,
    ) -> Result<bool, RuntimeError> {
        loop {
            if self.frames.len() <= min_frame_len {
                return Ok(false);
            }

            let current_frame_index = self.frames.len() - 1;

            let Some(handler_index) = self.exception_handlers.iter().rposition(|handler| {
                handler.finally_ip.is_some()
                    && handler.frame_index >= min_frame_len
                    && handler.frame_index <= current_frame_index
            }) else {
                self.close_current_frame_for_exception()?;
                continue;
            };

            let handler_frame = self.exception_handlers[handler_index].frame_index;

            while self.frames.len() > handler_frame + 1 {
                self.close_current_frame_for_exception()?;
            }

            if self.frames.len() <= min_frame_len {
                return Ok(false);
            }

            let finally_ip = self.exception_handlers[handler_index]
                .finally_ip
                .ok_or(RuntimeError::InvalidFunction)?;
            let stack_height = self.exception_handlers[handler_index].stack_height;

            // Les handlers plus internes (catch seuls) sont abandonnés avec
            // celui-ci : on quitte leur région protégée.
            self.exception_handlers.truncate(handler_index);
            self.restore_exception_stack(stack_height)?;

            self.push(Self::cancellation_value());
            self.push(Value::Boolean(true));
            self.current_frame_mut()?.ip = finally_ip;

            return Ok(true);
        }
    }
}

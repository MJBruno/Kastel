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
                super::RunStatus::Completed => return Ok(()),
                super::RunStatus::Yielded => continue,
                super::RunStatus::Waiting => {
                    return Err(RuntimeError::ChannelRecvOutsideTask);
                }
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn run_without_gc(&mut self) -> Result<(), RuntimeError> {
        loop {
            match self.run_internal(false, None)? {
                super::RunStatus::Completed => return Ok(()),
                super::RunStatus::Yielded => continue,
                super::RunStatus::Waiting => {
                    return Err(RuntimeError::ChannelRecvOutsideTask);
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
            if let Some(budget) = instruction_budget {
                if instructions >= budget {
                    return Ok(super::RunStatus::Yielded);
                }
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
                x if x == OpCode::Loop as u8 => {
                    self.loop_back()?;
                    Ok(false)
                }

                x if x == OpCode::AddLocalConst as u8 => {
                    self.add_local_const()?;
                    Ok(false)
                }

                x if x == OpCode::LessLocalConst as u8 => {
                    self.less_local_const()?;
                    Ok(false)
                }

                x if x == OpCode::JumpIfFalsePop as u8 => {
                    self.jump_if_false_pop()?;
                    Ok(false)
                }

                x if x == OpCode::LessLocalConstJump as u8 => {
                    self.less_local_const_jump()?;
                    Ok(false)
                }

                x if x == OpCode::LoopLessAddLocalConst as u8 => {
                    self.loop_less_add_local_const()?;
                    Ok(false)
                }

                x if x == OpCode::AddLocalLocal as u8 => {
                    self.add_local_local()?;
                    Ok(false)
                }

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
        match error {
            RuntimeError::Thrown(value) => self.propagate_thrown(value),

            error => {
                let value = self.runtime_error_value(&error)?;

                if self.propagate_thrown(value)? {
                    Ok(true)
                } else {
                    Err(error)
                }
            }
        }
    }

    pub(crate) fn propagate_runtime_error_until(
        &mut self,
        error: RuntimeError,
        min_frame_len: usize,
    ) -> Result<bool, RuntimeError> {
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
            | RuntimeError::TaskDeadlock
            | RuntimeError::YieldOutsideTask
            | RuntimeError::ChannelRecvOutsideTask => {
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

    pub(crate) fn propagate_thrown(&mut self, value: Value) -> Result<bool, RuntimeError> {
        self.propagate_thrown_until(value, 0)
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

            let matches = Self::exception_matches_catch_type(
                self.exception_handlers[handler_index].catch_type.as_deref(),
                &value,
            );
            let catch_ip = self.exception_handlers[handler_index].catch_ip;
            let finally_ip = self.exception_handlers[handler_index].finally_ip;
            let stack_height = self.exception_handlers[handler_index].stack_height;

            self.restore_exception_stack(stack_height)?;

            if matches {
                if let Some(catch_ip) = catch_ip {
                    self.exception_handlers[handler_index].catch_ip = None;

                    self.push(value);

                    self.current_frame_mut()?.ip = catch_ip;

                    return Ok(true);
                }

                self.exception_handlers.remove(handler_index);

                if let Some(finally_ip) = finally_ip {
                    self.pending_exception = Some(super::PendingException {
                        value,
                        rethrow: true,
                    });

                    self.current_frame_mut()?.ip = finally_ip;

                    return Ok(true);
                }

                self.close_current_frame_for_exception()?;
                continue;
            }

            // Le type du catch ne correspond pas. Le handler ne doit pas
            // bloquer la propagation : on le retire et on cherche un handler
            // extérieur. Son `finally`, s'il existe, doit toutefois toujours
            // être exécuté avant de poursuivre la propagation.
            self.exception_handlers.remove(handler_index);

            if let Some(finally_ip) = finally_ip {
                self.pending_exception = Some(super::PendingException {
                    value,
                    rethrow: true,
                });

                self.current_frame_mut()?.ip = finally_ip;

                return Ok(true);
            }

            // Pas de finally : chercher le prochain handler dans ce frame ou
            // dans un frame appelant.
            continue;
        }
    }
}

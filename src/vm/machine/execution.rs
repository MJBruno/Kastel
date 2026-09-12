use super::VirtualMachine;

use crate::error::runtime_error::RuntimeError;
use crate::runtime::gc;
use crate::runtime::value::Value;

impl VirtualMachine {
    pub fn run(&mut self) -> Result<(), RuntimeError> {
        let mut gc_check_counter = 0usize;

        loop {
            if cfg!(feature = "debug_trace") {
                self.debug_machine()?;
            }

            gc_check_counter += 1;

            if gc_check_counter >= 256 {
                gc_check_counter = 0;

                if gc::should_collect() {
                    self.collect_garbage();
                }
            }

            let instruction = self.read_byte()?;

            #[cfg(feature = "profile")]
            self.profile_instruction(instruction);

            let result = match instruction {
                // ========================================================
                // HOT DISPATCH
                // ========================================================

                30 => {
                    self.loop_back()?;
                    Ok(false)
                }

                63 => {
                    self.add_local_const()?;
                    Ok(false)
                }

                64 => {
                    self.less_local_const()?;
                    Ok(false)
                }

                66 => {
                    self.less_local_const_jump()?;
                    Ok(false)
                }

                67 => {
                    self.loop_less_add_local_const()?;
                    Ok(false)
                }

                68 => {
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
                    return Ok(());
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
            RuntimeError::Thrown(value) => {
                self.propagate_thrown_until(value, min_frame_len)
            }

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

    fn runtime_error_value(
        &self,
        error: &RuntimeError,
    ) -> Result<Value, RuntimeError> {
        match error {
            RuntimeError::TypeError
            | RuntimeError::DivisionByZero
            | RuntimeError::WrongArgumentCount { .. }
            | RuntimeError::NotCallable
            | RuntimeError::NativeError
            | RuntimeError::IndexOutOfBounds
            | RuntimeError::InterfaceMethodMissing { .. }
            | RuntimeError::InterfaceMethodArityMismatch { .. }
            | RuntimeError::ArrayIndexNotInteger
            | RuntimeError::ArrayIndexOutOfBounds { .. }
            | RuntimeError::NotIndexable
            | RuntimeError::NotObject
            | RuntimeError::ModuleError(_)
            | RuntimeError::ObjectFieldNotFound { .. }
            | RuntimeError::NotIterable
            | RuntimeError::IteratorExhausted
            | RuntimeError::InvalidShiftAmount => {
                Ok(Value::new_string(error.to_string()))
            }

            RuntimeError::Thrown(value) => Ok(value.clone()),

            RuntimeError::WithLocation { source, .. } => {
                self.runtime_error_value(source)
            }

            RuntimeError::StackUnderflow
            | RuntimeError::InvalidOpcode(_)
            | RuntimeError::InvalidFunction => Err(error.clone()),
        }
    }

    pub(crate) fn propagate_thrown(
        &mut self,
        value: Value,
    ) -> Result<bool, RuntimeError> {
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

            let Some(handler_index) = self
                .exception_handlers
                .iter()
                .rposition(|handler| {
                    handler.frame_index >= min_frame_len
                        && handler.frame_index <= current_frame_index
                })
            else {
                self.close_current_frame_for_exception()?;
                continue;
            };

            let handler_frame =
                self.exception_handlers[handler_index].frame_index;

            while self.frames.len() > handler_frame + 1 {
                self.close_current_frame_for_exception()?;
            }

            if self.frames.len() <= min_frame_len {
                return Ok(false);
            }

            let catch_ip = self.exception_handlers[handler_index].catch_ip;
            let finally_ip =
                self.exception_handlers[handler_index].finally_ip;
            let stack_height =
                self.exception_handlers[handler_index].stack_height;

            self.restore_exception_stack(stack_height)?;

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
        }
    }
}
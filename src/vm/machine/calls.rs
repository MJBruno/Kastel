use super::bytecode::frame_closure;
use super::{CallFrame, VirtualMachine};

use crate::error::runtime_error::RuntimeError;
use crate::runtime::gc_handle::Gc;
use crate::runtime::object::Object;
use crate::runtime::value::Value;

impl VirtualMachine {
    pub(crate) fn call(
        &mut self,
        closure: Gc<Object>,
        arg_count: usize,
    ) -> Result<(), RuntimeError> {
        let arity = {
            let closure = frame_closure(&closure);
            closure.function.arity
        };

        if arg_count != arity {
            return Err(RuntimeError::WrongArgumentCount {
                expected: arity,
                found: arg_count,
            });
        }

        let required = arg_count
            .checked_add(1)
            .ok_or(RuntimeError::InvalidFunction)?;

        if self.stack.len() < required {
            return Err(RuntimeError::StackUnderflow);
        }

        let callee_index = self.stack.len() - required;
        let current_frame = self.current_frame()?;

        if callee_index < current_frame.slot_start {
            return Err(RuntimeError::InvalidFunction);
        }

        self.frames.push(CallFrame {
            closure,
            ip: 0,
            slot_start: callee_index,
        });

        Ok(())
    }

    pub(crate) fn execute_call(&mut self, arg_count: usize) -> Result<(), RuntimeError> {
        let required = arg_count
            .checked_add(1)
            .ok_or(RuntimeError::InvalidFunction)?;

        if self.stack.len() < required {
            return Err(RuntimeError::StackUnderflow);
        }

        let callee_index = self.stack.len() - required;
        let current_frame = self.current_frame()?;

        if callee_index < current_frame.slot_start {
            return Err(RuntimeError::InvalidFunction);
        }

        let callee = self
            .stack
            .get(callee_index)
            .cloned()
            .ok_or(RuntimeError::StackUnderflow)?;

        match callee {
            Value::Object(handle) => {
                let bound = {
                    let object = handle.borrow();

                    match &*object {
                        Object::BoundMethod { method, receiver } => {
                            Some((method.clone(), receiver.clone()))
                        }

                        Object::Closure(_) => None,

                        _ => None,
                    }
                };

                if let Some((method, receiver)) = bound {
                    self.stack[callee_index] = Value::Object(method);
                    self.stack.insert(callee_index + 1, receiver);

                    let bound_arg_count = arg_count
                        .checked_add(1)
                        .ok_or(RuntimeError::InvalidFunction)?;

                    self.execute_call(bound_arg_count)?;
                    return Ok(());
                }

                if matches!(&*handle.borrow(), Object::Closure(_)) {
                    self.call(handle, arg_count)?;
                    return Ok(());
                }

                Err(RuntimeError::NotCallable)
            }

            Value::NativeFunction(native) => {
                let args_start = callee_index
                    .checked_add(1)
                    .ok_or(RuntimeError::InvalidFunction)?;

                let args = self
                    .stack
                    .get(args_start..)
                    .ok_or(RuntimeError::StackUnderflow)?
                    .to_vec();

                let result = native(&args)?;

                self.stack.truncate(callee_index);
                self.push(result);

                Ok(())
            }

            _ => Err(RuntimeError::NotCallable),
        }
    }

    pub(crate) fn invoke_sync(
        &mut self,
        callee: Value,
        arguments: &[Value],
    ) -> Result<Value, RuntimeError> {
        let base_stack_len = self.stack.len();
        let base_frame_len = self.frames.len();

        self.push(callee);

        for argument in arguments {
            self.push(argument.clone());
        }

        self.execute_call(arguments.len())?;

        while self.frames.len() > base_frame_len {
            let instruction = self.read_byte()?;

            match self.dispatch(instruction) {
                Ok(true) => {
                    self.stack.truncate(base_stack_len);
                    return Err(RuntimeError::InvalidFunction);
                }

                Ok(false) => {}

                Err(error) => {
                    match self.propagate_runtime_error_until(error.clone(), base_frame_len) {
                        Ok(true) => {}

                        Ok(false) => {
                            self.stack.truncate(base_stack_len);
                            return Err(error);
                        }

                        Err(propagation_error) => {
                            self.stack.truncate(base_stack_len);
                            return Err(propagation_error);
                        }
                    }
                }
            }
        }

        if self.stack.len() <= base_stack_len {
            self.stack.truncate(base_stack_len);
            return Err(RuntimeError::InvalidFunction);
        }

        let result = self.pop()?;

        self.stack.truncate(base_stack_len);

        Ok(result)
    }

    pub(crate) fn execute_return(&mut self) -> Result<(), RuntimeError> {
        let frame = self
            .frames
            .last()
            .cloned()
            .ok_or(RuntimeError::InvalidFunction)?;

        let minimum_stack_len = frame
            .slot_start
            .checked_add(2)
            .ok_or(RuntimeError::InvalidFunction)?;

        if self.stack.len() < minimum_stack_len {
            return Err(RuntimeError::InvalidFunction);
        }

        let result = self.pop()?;

        self.close_upvalues(frame.slot_start)?;
        self.remove_current_frame_handlers();

        self.frames.pop().ok_or(RuntimeError::InvalidFunction)?;

        self.stack.truncate(frame.slot_start);
        self.prune_exception_handlers();

        if !self.frames.is_empty() {
            self.push(result);
        }

        Ok(())
    }
}

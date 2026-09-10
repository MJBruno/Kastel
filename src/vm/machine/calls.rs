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
            let frame_closure = frame_closure(&closure);
            frame_closure.function.arity
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

        let callee = self.stack[callee_index].clone();

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

                    self.execute_call(arg_count + 1)?;

                    return Ok(());
                }

                let is_closure = {
                    let object = handle.borrow();

                    matches!(&*object, Object::Closure(_))
                };

                if is_closure {
                    self.call(handle, arg_count)?;
                    return Ok(());
                }

                Err(RuntimeError::NotCallable)
            }

            Value::NativeFunction(native) => {
                let args_start = callee_index + 1;

                let args = self.stack[args_start..].to_vec();

                let result = native(&args)?;

                self.stack.truncate(callee_index);
                self.push(result);

                Ok(())
            }

            _ => Err(RuntimeError::NotCallable),
        }
    }

    // ============================================================
    //            APPEL SYNCHRONE D'UNE VALEUR CALLABLE
    // ============================================================

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

            if self.dispatch(instruction)? {
                return Err(RuntimeError::InvalidFunction);
            }
        }

        if self.stack.len() <= base_stack_len {
            return Err(RuntimeError::InvalidFunction);
        }

        let result = self.pop()?;

        if self.stack.len() != base_stack_len {
            self.stack.truncate(base_stack_len);
        }

        Ok(result)
    }

    pub(crate) fn execute_return(&mut self) -> Result<(), RuntimeError> {
        let slot_start = self
            .frames
            .last()
            .ok_or(RuntimeError::InvalidFunction)?
            .slot_start;

        if slot_start > self.stack.len() {
            return Err(RuntimeError::InvalidFunction);
        }

        let result = self.pop()?;

        self.close_upvalues(slot_start)?;

        self.frames.pop().ok_or(RuntimeError::InvalidFunction)?;

        self.stack.truncate(slot_start);

        if self.frames.is_empty() {
            return Ok(());
        }

        self.push(result);

        Ok(())
    }
}

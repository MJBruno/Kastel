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

    pub(crate) fn execute_call(
        &mut self,
        arg_count: usize,
    ) -> Result<(), RuntimeError> {
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
                let is_closure = matches!(
                    &*handle.borrow(),
                    Object::Closure(_)
                );

                if is_closure {
                    self.call(handle, arg_count)
                } else {
                    Err(RuntimeError::NotCallable)
                }
            }

            Value::NativeFunction(function) => {
                let args_start = self.stack.len() - arg_count;
                let args = self.stack[args_start..].to_vec();

                let result = function(&args)?;

                // Supprime le callee et les arguments.
                self.stack.truncate(callee_index);

                self.push(result);

                Ok(())
            }

            _ => Err(RuntimeError::NotCallable),
        }
    }

    pub(crate) fn execute_return(&mut self) -> Result<(), RuntimeError> {
        let result = self.pop()?;

        let frame = self
            .frames
            .pop()
            .ok_or(RuntimeError::InvalidFunction)?;

        if frame.slot_start > self.stack.len() {
            return Err(RuntimeError::InvalidFunction);
        }

        // IMPORTANT :
        // les upvalues doivent être fermées avant de supprimer
        // les slots du frame de la stack.
        self.close_upvalues(frame.slot_start);

        self.stack.truncate(frame.slot_start);

        if self.frames.is_empty() {
            return Ok(());
        }

        self.push(result);

        Ok(())
    }
}
use std::rc::Rc;

use super::bytecode::frame_closure;
use super::{CallFrame, MAX_CALL_DEPTH, MAX_NATIVE_DEPTH, VirtualMachine};

use crate::error::runtime_error::RuntimeError;
use crate::runtime::gc;
use crate::runtime::gc_handle::Gc;
use crate::runtime::object::Object;
use crate::runtime::value::Value;

impl VirtualMachine {
    // ============================================================
    // CALL
    // ============================================================

    pub(crate) fn call(
        &mut self,
        closure: Gc<Object>,
        arg_count: usize,
    ) -> Result<(), RuntimeError> {
        let (arity, local_count, chunk) = {
            let closure_ref = frame_closure(&closure);

            (
                closure_ref.function.arity,
                closure_ref.function.local_count as usize,
                Rc::clone(&closure_ref.function.chunk),
            )
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

        if self.frames.len() >= MAX_CALL_DEPTH {
            return Err(RuntimeError::StackOverflow {
                limit: MAX_CALL_DEPTH,
            });
        }

        self.frames.push(CallFrame {
            closure,
            chunk,
            ip: 0,
            slot_start: callee_index,
            local_count,
            hot_loop_cache: None,
        });
        Ok(())
    }

    // ============================================================
    // EXECUTE CALL
    // ============================================================

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
                    let Some(method) = method else {
                        return Err(RuntimeError::InvalidFunction);
                    };

                    self.stack[callee_index] = Value::Object(method);

                    self.stack.insert(callee_index + 1, receiver);

                    let bound_arg_count = arg_count
                        .checked_add(1)
                        .ok_or(RuntimeError::InvalidFunction)?;

                    self.execute_call(bound_arg_count)?;

                    return Ok(());
                }

                // Fonction surchargée : on choisit la surcharge dont le nombre de
                // paramètres égale celui des arguments, puis c'est un appel
                // ordinaire de cette fermeture.
                let overloads = {
                    let object = handle.borrow();

                    match &*object {
                        Object::Overloads { functions, .. } => Some(functions.clone()),
                        _ => None,
                    }
                };

                if let Some(functions) = overloads {
                    let selected = functions
                        .iter()
                        .find(|function| Self::function_arity(function) == Some(arg_count))
                        .cloned();

                    return match selected {
                        Some(Value::Object(closure)) => {
                            self.stack[callee_index] = Value::Object(closure.clone());
                            self.call(closure, arg_count)
                        }

                        _ => {
                            // Aucune surcharge à `arg_count` paramètres :
                            // erreur d'arité, avec l'arité déclarée la plus petite.
                            let expected = functions
                                .iter()
                                .filter_map(Self::function_arity)
                                .min()
                                .unwrap_or(0);

                            Err(RuntimeError::WrongArgumentCount {
                                expected,
                                found: arg_count,
                            })
                        }
                    };
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
                let args_start = callee_index
                    .checked_add(1)
                    .ok_or(RuntimeError::InvalidFunction)?;

                let args_end = args_start
                    .checked_add(arg_count)
                    .ok_or(RuntimeError::InvalidFunction)?;

                let args = self
                    .stack
                    .get(args_start..args_end)
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

    // ============================================================
    // SYNCHRONOUS INVOCATION
    // ============================================================

    /// Appelle `callee` depuis du code natif et attend son résultat.
    ///
    /// Chaque niveau d'imbrication consomme de la pile Rust : au-delà de
    /// `MAX_NATIVE_DEPTH`, `StackOverflow` plutôt qu'un débordement natif.
    pub(crate) fn invoke_sync(
        &mut self,
        callee: Value,
        arguments: &[Value],
    ) -> Result<Value, RuntimeError> {
        if self.native_depth >= MAX_NATIVE_DEPTH {
            return Err(RuntimeError::StackOverflow {
                limit: MAX_NATIVE_DEPTH,
            });
        }

        self.native_depth += 1;
        let result = self.invoke_sync_inner(callee, arguments);
        self.native_depth -= 1;

        result
    }

    fn invoke_sync_inner(
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
            if cfg!(feature = "debug_trace") {
                self.debug_machine()?;
            }

            if gc::should_collect() {
                self.collect_garbage();
            }

            let (line, column) = self.current_position()?;
            self.current_line = line;
            self.current_column = column;

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

                        Err(error) => {
                            self.stack.truncate(base_stack_len);
                            return Err(error);
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

    // ============================================================
    // RETURN
    // ============================================================

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

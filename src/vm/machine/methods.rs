use super::bytecode::frame_closure;
use super::VirtualMachine;

use crate::{
    error::runtime_error::RuntimeError,
    runtime::{
        gc_handle::Gc,
        object::Object,
        value::Value,
    },
    stdlib::{array, dict},
};

impl VirtualMachine {
    // ============================================================
    //                     METHOD RESOLUTION
    // ============================================================

    pub(crate) fn find_class_method_from(
        class: Gc<Object>,
        name: &str,
    ) -> Option<Value> {
        Self::find_method_in_hierarchy(Some(class), name)
    }

    pub(crate) fn find_base_method(
        class: Gc<Object>,
        name: &str,
    ) -> Option<Value> {
        let parent = {
            let object = class.borrow();

            match &*object {
                Object::Class { superclass, .. } => superclass.clone(),
                _ => None,
            }
        };

        Self::find_method_in_hierarchy(parent, name)
    }

    fn find_method_in_hierarchy(
        mut current: Option<Gc<Object>>,
        name: &str,
    ) -> Option<Value> {
        while let Some(handle) = current {
            let object = handle.borrow();

            match &*object {
                Object::Class {
                    methods,
                    superclass,
                    ..
                } => {
                    if let Some(method) = methods.get(name) {
                        return Some(method.clone());
                    }

                    current = superclass.clone();
                }

                _ => return None,
            }
        }

        None
    }

    // ============================================================
    //                     INVOKE METHOD
    // ============================================================

    pub(crate) fn op_invoke_method(
        &mut self,
        method_constant: usize,
        arg_count: usize,
    ) -> Result<(), RuntimeError> {
        let method_constant = method_constant
            .try_into()
            .map_err(|_| RuntimeError::InvalidFunction)?;

        let method_value = self.read_constant(method_constant)?;

        let method_name = method_value
            .as_string_value()
            .ok_or(RuntimeError::TypeError)?;

        let required = arg_count
            .checked_add(1)
            .ok_or(RuntimeError::InvalidFunction)?;

        if self.stack.len() < required {
            return Err(RuntimeError::StackUnderflow);
        }

        let receiver_index = self.stack.len() - required;
        let receiver = self.stack[receiver_index].clone();
        let args = self.stack[receiver_index..].to_vec();

        self.stack.truncate(receiver_index);

        if method_name == "to_iterator" {
            if arg_count != 0 {
                return Err(RuntimeError::WrongArgumentCount {
                    expected: 0,
                    found: arg_count,
                });
            }

            self.push(receiver.to_iterator()?);
            return Ok(());
        }

        let result = match &receiver {
            Value::Range { .. } => {
                let iterator = receiver.to_iterator()?;

                let mut iterator_args = args.clone();
                iterator_args[0] = iterator;

                self.invoke_iterator_method(
                    &method_name,
                    &iterator_args,
                )?
            }

            Value::Object(handle) => {
                let object_kind = {
                    let object = handle.borrow();

                    match &*object {
                        Object::Instance { .. } => 0,
                        Object::Iterator(_) => 1,
                        Object::String(_) => 2,
                        Object::Array(_) => 3,
                        Object::Dict(_) => 4,
                        _ => 5,
                    }
                };

                match object_kind {
                    0 => {
                        let callable =
                            receiver.get_property(&method_name)?;

                        self.push(callable);

                        for argument in args.iter().skip(1) {
                            self.push(argument.clone());
                        }

                        self.execute_call(arg_count)?;

                        return Ok(());
                    }

                    1 => {
                        self.invoke_iterator_method(
                            &method_name,
                            &args,
                        )?
                    }

                    2 => {
                        match crate::stdlib::string::dispatch_method(
                            &method_name,
                            &args,
                        )? {
                            Some(result) => result,

                            None => {
                                return Err(
                                    RuntimeError::ObjectFieldNotFound {
                                        name: method_name,
                                        suggestion: None,
                                    },
                                );
                            }
                        }
                    }

                    3 => {
                        if let Some(result) =
                            array::dispatch_method(&method_name, &args)?
                        {
                            result
                        } else {
                            self.invoke_array_functional(
                                &method_name,
                                &args,
                            )?
                        }
                    }

                    4 => {
                        match dict::dispatch_method(
                            &method_name,
                            &args,
                        )? {
                            Some(result) => result,

                            None => {
                                return Err(
                                    RuntimeError::ObjectFieldNotFound {
                                        name: method_name,
                                        suggestion: None,
                                    },
                                );
                            }
                        }
                    }

                    _ => {
                        return Err(RuntimeError::ObjectFieldNotFound {
                            name: method_name,
                            suggestion: None,
                        });
                    }
                }
            }

            _ => return Err(RuntimeError::NotObject),
        };

        self.push(result);
        Ok(())
    }

    // ============================================================
    //                     BASE METHOD
    // ============================================================

    pub(crate) fn op_invoke_base_method(
        &mut self,
        method_constant: usize,
        arg_count: usize,
    ) -> Result<(), RuntimeError> {
        let method_constant = u8::try_from(method_constant)
            .map_err(|_| RuntimeError::InvalidFunction)?;

        let method_value = self.read_constant(method_constant)?;

        let method_name = method_value
            .as_string_value()
            .ok_or(RuntimeError::TypeError)?;

        let frame = self
            .frames
            .last()
            .cloned()
            .ok_or(RuntimeError::InvalidFunction)?;

        let this_index = frame
            .slot_start
            .checked_add(1)
            .ok_or(RuntimeError::InvalidFunction)?;

        if this_index >= self.stack.len() {
            return Err(RuntimeError::StackUnderflow);
        }

        let this_value = self
            .stack
            .get(this_index)
            .cloned()
            .ok_or(RuntimeError::StackUnderflow)?;

        if self.stack.len() < arg_count {
            return Err(RuntimeError::StackUnderflow);
        }

        let args_start = self.stack.len() - arg_count;

        let args = self
            .stack
            .get(args_start..)
            .ok_or(RuntimeError::StackUnderflow)?
            .to_vec();

        self.stack.truncate(args_start);

        let owner_class = {
            let closure = frame_closure(&frame.closure);

            closure
                .owner_class
                .clone()
                .ok_or(RuntimeError::TypeError)?
        };

        let method =
            Self::find_base_method(owner_class, &method_name).ok_or(
                RuntimeError::ObjectFieldNotFound {
                    name: method_name,
                    suggestion: None,
                },
            )?;

        self.push(method);
        self.push(this_value);

        for argument in args {
            self.push(argument);
        }

        self.execute_call(arg_count + 1)?;

        Ok(())
    }

    // ============================================================
    //                  ARRAY FUNCTIONAL METHODS
    // ============================================================

    fn invoke_array_functional(
        &mut self,
        method: &str,
        args: &[Value],
    ) -> Result<Value, RuntimeError> {
        match method {
            "map" => {
                if args.len() != 2 {
                    return Err(RuntimeError::WrongArgumentCount {
                        expected: 2,
                        found: args.len(),
                    });
                }

                let elements = Self::array_snapshot(&args[0])?;
                let callback = args[1].clone();

                let mut result = Vec::with_capacity(elements.len());

                for element in elements {
                    result.push(
                        self.invoke_sync(
                            callback.clone(),
                            &[element],
                        )?,
                    );
                }

                Ok(Value::new_array(result))
            }

            "filter" => {
                if args.len() != 2 {
                    return Err(RuntimeError::WrongArgumentCount {
                        expected: 2,
                        found: args.len(),
                    });
                }

                let elements = Self::array_snapshot(&args[0])?;
                let callback = args[1].clone();

                let mut result = Vec::new();

                for element in elements {
                    let keep = self.invoke_sync(
                        callback.clone(),
                        std::slice::from_ref(&element),
                    )?;

                    if keep.is_truthy() {
                        result.push(element);
                    }
                }

                Ok(Value::new_array(result))
            }

            "reduce" => {
                if args.len() != 3 {
                    return Err(RuntimeError::WrongArgumentCount {
                        expected: 3,
                        found: args.len(),
                    });
                }

                let elements = Self::array_snapshot(&args[0])?;
                let callback = args[1].clone();
                let mut accumulator = args[2].clone();

                for element in elements {
                    accumulator = self.invoke_sync(
                        callback.clone(),
                        &[accumulator, element],
                    )?;
                }

                Ok(accumulator)
            }

            "any" => {
                if args.len() != 2 {
                    return Err(RuntimeError::WrongArgumentCount {
                        expected: 2,
                        found: args.len(),
                    });
                }

                let elements = Self::array_snapshot(&args[0])?;
                let callback = args[1].clone();

                for element in elements {
                    let value = self.invoke_sync(
                        callback.clone(),
                        &[element],
                    )?;

                    if value.is_truthy() {
                        return Ok(Value::Boolean(true));
                    }
                }

                Ok(Value::Boolean(false))
            }

            "all" => {
                if args.len() != 2 {
                    return Err(RuntimeError::WrongArgumentCount {
                        expected: 2,
                        found: args.len(),
                    });
                }

                let elements = Self::array_snapshot(&args[0])?;
                let callback = args[1].clone();

                for element in elements {
                    let value = self.invoke_sync(
                        callback.clone(),
                        &[element],
                    )?;

                    if !value.is_truthy() {
                        return Ok(Value::Boolean(false));
                    }
                }

                Ok(Value::Boolean(true))
            }

            _ => Err(RuntimeError::ObjectFieldNotFound {
                name: method.to_string(),
                suggestion: None,
            }),
        }
    }

    fn array_snapshot(
        value: &Value,
    ) -> Result<Vec<Value>, RuntimeError> {
        match value {
            Value::Object(handle) => {
                let object = handle.borrow();

                match &*object {
                    Object::Array(array) => Ok(array.clone()),
                    _ => Err(RuntimeError::TypeError),
                }
            }

            _ => Err(RuntimeError::TypeError),
        }
    }
}
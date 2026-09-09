use super::VirtualMachine;

use crate::{
    error::runtime_error::RuntimeError,
    runtime::object::Object,
    runtime::value::Value,
    stdlib::{array, dict},
};

impl VirtualMachine {
    // ============================================================
    //                      ARRAY CREATION
    // ============================================================

    pub(crate) fn op_array(&mut self, count: usize) -> Result<(), RuntimeError> {
        if self.stack.len() < count {
            return Err(RuntimeError::StackUnderflow);
        }

        let start = self.stack.len() - count;

        let values = self.stack[start..].to_vec();

        self.stack.truncate(start);

        self.push(Value::new_array(values));

        Ok(())
    }

    pub(crate) fn op_object(&mut self, pair_count: usize) -> Result<(), RuntimeError> {
        let total = pair_count
            .checked_mul(2)
            .ok_or(RuntimeError::InvalidFunction)?;

        if self.stack.len() < total {
            return Err(RuntimeError::StackUnderflow);
        }

        let start = self.stack.len() - total;

        let mut fields = Vec::with_capacity(pair_count);

        for index in 0..pair_count {
            let base = start + index * 2;

            let key = self.stack[base].clone();

            let value = self.stack[base + 1].clone();

            fields.push((key, value));
        }

        self.stack.truncate(start);

        self.push(Value::new_dict(fields));

        Ok(())
    }

    // ============================================================
    //                         INDEX
    // ============================================================

    pub(crate) fn op_get_index(&mut self) -> Result<(), RuntimeError> {
        if self.stack.len() < 2 {
            return Err(RuntimeError::StackUnderflow);
        }

        let len = self.stack.len();

        let collection = self.stack[len - 2].clone();

        let index = self.stack[len - 1].clone();

        let value = match &collection {
            Value::Object(handle) => {
                let object = handle.borrow();

                match &*object {
                    Object::Array(_) => {
                        let index = Self::array_index(index)?;

                        drop(object);

                        collection.array_get(index)?
                    }

                    Object::Dict(_) => {
                        drop(object);

                        collection.dict_get(&index)?
                    }

                    _ => {
                        return Err(RuntimeError::NotIndexable);
                    }
                }
            }

            _ => {
                return Err(RuntimeError::NotIndexable);
            }
        };

        self.stack.truncate(len - 2);
        self.push(value);

        Ok(())
    }

    pub(crate) fn op_set_index(&mut self) -> Result<(), RuntimeError> {
        if self.stack.len() < 3 {
            return Err(RuntimeError::StackUnderflow);
        }

        let len = self.stack.len();

        let collection = self.stack[len - 3].clone();

        let index = self.stack[len - 2].clone();

        let value = self.stack[len - 1].clone();

        match &collection {
            Value::Object(handle) => {
                let object = handle.borrow();

                match &*object {
                    Object::Array(_) => {
                        let index = Self::array_index(index)?;

                        drop(object);

                        collection.array_set(index, value)?;
                    }

                    Object::Dict(_) => {
                        drop(object);

                        collection.dict_set(&index, value)?;
                    }

                    _ => {
                        return Err(RuntimeError::NotIndexable);
                    }
                }
            }

            _ => {
                return Err(RuntimeError::NotIndexable);
            }
        }

        self.stack.truncate(len - 3);

        Ok(())
    }

    // ============================================================
    //                    ARRAY PRIMITIVES
    // ============================================================

    pub(crate) fn op_array_length(&mut self) -> Result<(), RuntimeError> {
        let array = self.peek()?.clone();

        let length = array.array_len()? as i64;

        self.pop()?;

        self.push(Value::Integer(length));

        Ok(())
    }

    pub(crate) fn op_array_push(&mut self) -> Result<(), RuntimeError> {
        if self.stack.len() < 2 {
            return Err(RuntimeError::StackUnderflow);
        }

        let len = self.stack.len();

        let array = self.stack[len - 2].clone();

        let value = self.stack[len - 1].clone();

        let length = array.array_push(value)?;

        self.stack.truncate(len - 2);

        self.push(Value::Integer(length as i64));

        Ok(())
    }

    pub(crate) fn op_array_pop(&mut self) -> Result<(), RuntimeError> {
        let array = self.peek()?.clone();

        let value = array.array_pop()?;

        self.pop()?;

        self.push(value);

        Ok(())
    }

    pub(crate) fn op_array_insert(&mut self) -> Result<(), RuntimeError> {
        if self.stack.len() < 3 {
            return Err(RuntimeError::StackUnderflow);
        }

        let len = self.stack.len();

        let array = self.stack[len - 3].clone();

        let index_value = self.stack[len - 2].clone();

        let value = self.stack[len - 1].clone();

        let index = Self::array_index(index_value)?;

        let new_length = array.array_insert(index, value)?;

        self.stack.truncate(len - 3);

        self.push(Value::Integer(new_length as i64));

        Ok(())
    }

    pub(crate) fn op_array_remove(&mut self) -> Result<(), RuntimeError> {
        if self.stack.len() < 2 {
            return Err(RuntimeError::StackUnderflow);
        }

        let len = self.stack.len();

        let array = self.stack[len - 2].clone();

        let index_value = self.stack[len - 1].clone();

        let index = Self::array_index(index_value)?;

        let value = array.array_remove(index)?;

        self.stack.truncate(len - 2);

        self.push(value);

        Ok(())
    }

    pub(crate) fn op_array_clear(&mut self) -> Result<(), RuntimeError> {
        let array = self.peek()?.clone();

        array.array_clear()?;

        self.pop()?;

        Ok(())
    }

    pub(crate) fn op_array_contains(&mut self) -> Result<(), RuntimeError> {
        if self.stack.len() < 2 {
            return Err(RuntimeError::StackUnderflow);
        }

        let len = self.stack.len();

        let array = self.stack[len - 2].clone();

        let value = self.stack[len - 1].clone();

        let contains = array.array_contains(&value)?;

        self.stack.truncate(len - 2);

        self.push(Value::Boolean(contains));

        Ok(())
    }

    // ============================================================
    //                     INVOKE METHOD
    // ============================================================

    pub(crate) fn op_invoke_method(
        &mut self,
        method_constant: usize,
        arg_count: usize,
    ) -> Result<(), RuntimeError> {
        let method_value = self.read_constant(method_constant.try_into().unwrap())?;

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
        // ============================================================
        //                     to_iterator()
        // ============================================================
        //
        // Conversion générique :
        //
        // Array    -> Iterator
        // Dict     -> Iterator
        // String   -> Iterator
        // Range    -> Iterator
        // Iterator -> lui-même
        //
        // ============================================================

        if method_name == "to_iterator" {
            if arg_count != 0 {
                return Err(RuntimeError::WrongArgumentCount {
                    expected: 0,
                    found: arg_count,
                });
            }

            let iterator = receiver.to_iterator()?;

            self.push(iterator);

            return Ok(());
        }
        let result = match &receiver {
            // ============================================================
            // RANGE
            // ============================================================
            //
            // range(5) produit initialement Value::Range.
            //
            // Pour les méthodes iterator(), on le convertit une seule fois
            // en Object::Iterator.
            //
            Value::Range { .. } => {
                let iterator = receiver.to_iterator()?;

                self.invoke_iterator_method(&method_name, &{
                    let mut iterator_args = args.clone();
                    iterator_args[0] = iterator;
                    iterator_args
                })?
            }

            // ============================================================
            // OBJECT
            // ============================================================
            Value::Object(handle) => {
                let object = handle.borrow();

                match &*object {
                    // ====================================================
                    // ITERATOR
                    // ====================================================
                    Object::Iterator(_) => {
                        drop(object);

                        self.invoke_iterator_method(&method_name, &args)?
                    }

                    // ====================================================
                    // STRING
                    // ====================================================
                    Object::String(_) => {
                        drop(object);

                        match crate::stdlib::string::dispatch_method(&method_name, &args)? {
                            Some(result) => result,

                            None => {
                                return Err(RuntimeError::ObjectFieldNotFound {
                                    name: method_name,
                                    suggestion: None,
                                });
                            }
                        }
                    }

                    // ====================================================
                    // ARRAY
                    // ====================================================
                    Object::Array(_) => {
                        drop(object);

                        if let Some(result) = array::dispatch_method(&method_name, &args)? {
                            result
                        } else {
                            self.invoke_array_functional(&method_name, &args)?
                        }
                    }

                    // ====================================================
                    // DICT
                    // ====================================================
                    Object::Dict(_) => {
                        drop(object);

                        match dict::dispatch_method(&method_name, &args)? {
                            Some(result) => result,

                            None => {
                                return Err(RuntimeError::ObjectFieldNotFound {
                                    name: method_name,
                                    suggestion: None,
                                });
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

            _ => {
                return Err(RuntimeError::NotObject);
            }
        };

        self.push(result);

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
                    result.push(self.invoke_sync(callback.clone(), &[element])?);
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
                    let keep =
                        self.invoke_sync(callback.clone(), std::slice::from_ref(&element))?;

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
                    accumulator = self.invoke_sync(callback.clone(), &[accumulator, element])?;
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
                    let value = self.invoke_sync(callback.clone(), &[element])?;

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
                    let value = self.invoke_sync(callback.clone(), &[element])?;

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

    fn array_snapshot(value: &Value) -> Result<Vec<Value>, RuntimeError> {
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

    // ============================================================
    //                         PROPERTY
    // ============================================================

    pub(crate) fn get_property(&mut self) -> Result<(), RuntimeError> {
        let constant = self.read_byte()?;

        let property = self.read_constant(constant)?;

        let name = property.as_string_value().ok_or(RuntimeError::TypeError)?;

        let object = self.peek()?.clone();

        let value = object.get_property(&name)?;

        self.pop()?;

        self.push(value);

        Ok(())
    }

    pub(crate) fn set_property(&mut self) -> Result<(), RuntimeError> {
        let constant = self.read_byte()?;

        let property = self.read_constant(constant)?;

        let name = property.as_string_value().ok_or(RuntimeError::TypeError)?;

        if self.stack.len() < 2 {
            return Err(RuntimeError::StackUnderflow);
        }

        let len = self.stack.len();

        let object = self.stack[len - 2].clone();

        let value = self.stack[len - 1].clone();

        object.set_property(&name, value)?;

        self.stack.truncate(len - 2);

        Ok(())
    }

    // ============================================================
    //                         HELPERS
    // ============================================================

    fn array_index(value: Value) -> Result<usize, RuntimeError> {
        match value {
            Value::Integer(index) if index >= 0 => {
                usize::try_from(index).map_err(|_| RuntimeError::TypeError)
            }

            Value::Float(index)
                if index.is_finite()
                    && index >= 0.0
                    && index.fract() == 0.0
                    && index <= usize::MAX as f64 =>
            {
                Ok(index as usize)
            }

            _ => Err(RuntimeError::ArrayIndexNotInteger),
        }
    }
}

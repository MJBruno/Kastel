use super::VirtualMachine;

use crate::error::runtime_error::RuntimeError;
use crate::runtime::object::Object;
use crate::runtime::value::Value;

impl VirtualMachine {
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

        for index in (start..self.stack.len()).step_by(2) {
            if !self.stack[index].is_string() {
                return Err(RuntimeError::TypeError);
            }
        }

        let mut fields = Vec::with_capacity(pair_count);

        for index in 0..pair_count {
            let base = start + index * 2;

            let key = self.stack[base]
                .as_string_value()
                .ok_or(RuntimeError::TypeError)?;

            let value = self.stack[base + 1].clone();

            fields.push((key, value));
        }

        self.stack.truncate(start);

        self.push(Value::new_dict(fields));

        Ok(())
    }

    // ------------------------------------------------------------
    // Indexation
    // ------------------------------------------------------------

    pub(crate) fn op_get_index(&mut self) -> Result<(), RuntimeError> {
        if self.stack.len() < 2 {
            return Err(RuntimeError::StackUnderflow);
        }

        let len = self.stack.len();

        let collection = self.stack[len - 2].clone();
        let index = self.stack[len - 1].clone();

        let value = match &collection {
            // Array[index]
            Value::Object(handle) => {
                let object = handle.borrow();

                match &*object {
                    Object::Array(_) => {
                        let index = Self::array_index(index)?;
                        drop(object);

                        collection.array_get(index)?
                    }

                    // Dict["key"]
                    Object::Dict(_) => {
                        let key = Self::dict_key(index)?;
                        drop(object);

                        collection.dict_get(&key)?
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
            // Array[index] = value
            Value::Object(handle) => {
                let object = handle.borrow();

                match &*object {
                    Object::Array(_) => {
                        let index = Self::array_index(index)?;
                        drop(object);

                        collection.array_set(index, value)?;
                    }

                    // Dict["key"] = value
                    Object::Dict(_) => {
                        let key = Self::dict_key(index)?;
                        drop(object);

                        collection.dict_set(&key, value)?;
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

    // ------------------------------------------------------------
    // Array
    // ------------------------------------------------------------

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

    // ------------------------------------------------------------
    // Properties
    // ------------------------------------------------------------

    pub(crate) fn get_property(&mut self) -> Result<(), RuntimeError> {
        let constant = self.read_byte()?;

        let property = self.read_constant(constant)?;

        let name = property
            .as_string_value()
            .ok_or(RuntimeError::TypeError)?;

        let object = self.peek()?.clone();

        let value = object.get_property(&name)?;

        self.pop()?;
        self.push(value);

        Ok(())
    }

    pub(crate) fn set_property(&mut self) -> Result<(), RuntimeError> {
        let constant = self.read_byte()?;

        let property = self.read_constant(constant)?;

        let name = property
            .as_string_value()
            .ok_or(RuntimeError::TypeError)?;

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

    // ------------------------------------------------------------
    // Index helpers
    // ------------------------------------------------------------

    fn array_index(value: Value) -> Result<usize, RuntimeError> {
        match value {
            Value::Integer(index) if index >= 0 => Ok(index as usize),

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

    fn dict_key(value: Value) -> Result<String, RuntimeError> {
        match value {
            Value::Object(handle) => {
                let object = handle.borrow();

                match &*object {
                    Object::String(value) => Ok(value.clone()),

                    _ => Err(RuntimeError::TypeError),
                }
            }

            _ => Err(RuntimeError::TypeError),
        }
    }
}
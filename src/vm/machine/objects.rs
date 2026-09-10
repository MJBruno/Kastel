use super::VirtualMachine;

use crate::{
    error::runtime_error::RuntimeError,
    runtime::{
        object::Object,
        value::Value,
    },
};

impl VirtualMachine {
    pub(crate) fn op_object(
        &mut self,
        pair_count: usize,
    ) -> Result<(), RuntimeError> {
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

            fields.push((
                self.stack[base].clone(),
                self.stack[base + 1].clone(),
            ));
        }

        self.stack.truncate(start);
        self.push(Value::new_dict(fields));

        Ok(())
    }

    pub(crate) fn op_get_dict_index(
        &mut self,
    ) -> Result<(), RuntimeError> {
        if self.stack.len() < 2 {
            return Err(RuntimeError::StackUnderflow);
        }

        let len = self.stack.len();
        let dict = self.stack[len - 2].clone();
        let index = self.stack[len - 1].clone();

        let value = match &dict {
            Value::Object(handle) => {
                let object = handle.borrow();

                if matches!(&*object, Object::Dict(_)) {
                    drop(object);
                    dict.dict_get(&index)?
                } else {
                    return Err(RuntimeError::NotIndexable);
                }
            }

            _ => return Err(RuntimeError::NotIndexable),
        };

        self.stack.truncate(len - 2);
        self.push(value);

        Ok(())
    }

    pub(crate) fn op_set_dict_index(
        &mut self,
    ) -> Result<(), RuntimeError> {
        if self.stack.len() < 3 {
            return Err(RuntimeError::StackUnderflow);
        }

        let len = self.stack.len();
        let dict = self.stack[len - 3].clone();
        let index = self.stack[len - 2].clone();
        let value = self.stack[len - 1].clone();

        match &dict {
            Value::Object(handle) => {
                let object = handle.borrow();

                if matches!(&*object, Object::Dict(_)) {
                    drop(object);
                    dict.dict_set(&index, value)?;
                } else {
                    return Err(RuntimeError::NotIndexable);
                }
            }

            _ => return Err(RuntimeError::NotIndexable),
        }

        self.stack.truncate(len - 3);

        Ok(())
    }
}
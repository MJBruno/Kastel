use super::VirtualMachine;
use crate::error::runtime_error::RuntimeError;

impl VirtualMachine {
    pub(crate) fn op_array(&mut self, count: usize) -> Result<(), RuntimeError> {
        if self.stack.len() < count {
            return Err(RuntimeError::StackUnderflow);
        }
        let start = self.stack.len() - count;
        let values = self.stack.drain(start..).collect::<Vec<_>>();
        self.push(crate::runtime::value::Value::new_array(values));
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
        let drained = self.stack.drain(start..).collect::<Vec<_>>();
        let mut fields = Vec::with_capacity(pair_count);
        let mut iter = drained.into_iter();
        while let (Some(key), Some(value)) = (iter.next(), iter.next()) {
            let key = key.as_string_value().ok_or(RuntimeError::TypeError)?;
            fields.push((key, value));
        }
        self.push(crate::runtime::value::Value::new_object(fields));
        Ok(())
    }

    pub(crate) fn op_get_index(&mut self) -> Result<(), RuntimeError> {
        let index = Self::array_index(self.pop()?)?;
        let array = self.pop()?;
        let value = array.array_get(index)?;
        self.push(value);
        Ok(())
    }

    pub(crate) fn op_set_index(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop()?;
        let index = Self::array_index(self.pop()?)?;
        let array = self.pop()?;
        array.array_set(index, value)?;
        Ok(())
    }

    pub(crate) fn op_array_length(&mut self) -> Result<(), RuntimeError> {
        let array = self.pop()?;
        self.push(crate::runtime::value::Value::Integer(
            array.array_len()? as i64
        ));
        Ok(())
    }

    pub(crate) fn op_array_push(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop()?;
        let array = self.pop()?;
        let length = array.array_push(value)?;
        self.push(crate::runtime::value::Value::Integer(length as i64));
        Ok(())
    }

    pub(crate) fn op_array_pop(&mut self) -> Result<(), RuntimeError> {
        let array = self.pop()?;
        self.push(array.array_pop()?);
        Ok(())
    }

    pub(crate) fn op_array_insert(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop()?;
        let index = Self::array_index(self.pop()?)?;
        let array = self.pop()?;
        let length = array.array_insert(index, value)?;
        self.push(crate::runtime::value::Value::Integer(length as i64));
        Ok(())
    }

    pub(crate) fn op_array_remove(&mut self) -> Result<(), RuntimeError> {
        let index = Self::array_index(self.pop()?)?;
        let array = self.pop()?;
        self.push(array.array_remove(index)?);
        Ok(())
    }

    pub(crate) fn op_array_clear(&mut self) -> Result<(), RuntimeError> {
        let array = self.pop()?;
        array.array_clear()?;
        Ok(())
    }

    pub(crate) fn op_array_contains(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop()?;
        let array = self.pop()?;
        self.push(crate::runtime::value::Value::Boolean(
            array.array_contains(&value)?,
        ));
        Ok(())
    }

    pub(crate) fn get_property(&mut self) -> Result<(), RuntimeError> {
        let constant = self.read_byte()?;
        let property = self.read_constant(constant)?;
        let name = property.as_string_value().ok_or(RuntimeError::TypeError)?;
        let object = self.pop()?;
        self.push(object.get_property(&name)?);
        Ok(())
    }

    pub(crate) fn set_property(&mut self) -> Result<(), RuntimeError> {
        let constant = self.read_byte()?;
        let property = self.read_constant(constant)?;
        let name = property.as_string_value().ok_or(RuntimeError::TypeError)?;
        let value = self.pop()?;
        let object = self.pop()?;
        object.set_property(&name, value)?;
        Ok(())
    }

    fn array_index(value: crate::runtime::value::Value) -> Result<usize, RuntimeError> {
        match value {
            crate::runtime::value::Value::Integer(index) if index >= 0 => Ok(index as usize),
            crate::runtime::value::Value::Float(index)
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

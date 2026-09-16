use super::VirtualMachine;

use crate::{error::runtime_error::RuntimeError, runtime::value::Value};

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

    pub(crate) fn op_get_array_index(&mut self) -> Result<(), RuntimeError> {
        if self.stack.len() < 2 {
            return Err(RuntimeError::StackUnderflow);
        }

        let len = self.stack.len();
        let array = self.stack[len - 2].clone();
        let index = Self::array_index(self.stack[len - 1].clone())?;

        let value = array.array_get(index)?;

        self.stack.truncate(len - 2);
        self.push(value);

        Ok(())
    }

    pub(crate) fn op_set_array_index(&mut self) -> Result<(), RuntimeError> {
        if self.stack.len() < 3 {
            return Err(RuntimeError::StackUnderflow);
        }

        let len = self.stack.len();
        let array = self.stack[len - 3].clone();
        let index = Self::array_index(self.stack[len - 2].clone())?;
        let value = self.stack[len - 1].clone();

        array.array_set(index, value)?;

        self.stack.truncate(len - 3);

        Ok(())
    }

    pub(crate) fn op_array_length(&mut self) -> Result<(), RuntimeError> {
        let value = self.peek()?.clone();

        // `.length` est compilé de façon générique (voir
        // `compile_array_member`) pour n'importe quel receveur ; on
        // essaie donc Array puis, à défaut, Tuple.
        let length = match value.array_len() {
            Ok(length) => length,
            Err(_) => value.tuple_len()?,
        } as i64;

        self.pop()?;
        self.push(Value::Integer(length));

        Ok(())
    }

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

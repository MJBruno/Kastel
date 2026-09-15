use super::VirtualMachine;

use crate::{error::runtime_error::RuntimeError, runtime::value::Value};

impl VirtualMachine {
    pub(crate) fn op_tuple(&mut self, count: usize) -> Result<(), RuntimeError> {
        if self.stack.len() < count {
            return Err(RuntimeError::StackUnderflow);
        }

        let start = self.stack.len() - count;
        let values = self.stack[start..].to_vec();

        self.stack.truncate(start);
        self.push(Value::new_tuple(values));

        Ok(())
    }

    pub(crate) fn op_get_tuple_index(&mut self) -> Result<(), RuntimeError> {
        if self.stack.len() < 2 {
            return Err(RuntimeError::StackUnderflow);
        }

        let len = self.stack.len();
        let tuple = self.stack[len - 2].clone();
        let index = Self::tuple_index(self.stack[len - 1].clone())?;

        let value = tuple.tuple_get(index)?;

        self.stack.truncate(len - 2);
        self.push(value);

        Ok(())
    }

    fn tuple_index(value: Value) -> Result<usize, RuntimeError> {
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

use super::VirtualMachine;
use crate::error::runtime_error::RuntimeError;
use crate::runtime::value::Value;

impl VirtualMachine {
    pub(crate) fn op_get_iterator(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop()?;
        self.push(value.to_iterator()?);
        Ok(())
    }

    pub(crate) fn op_iterator_has_next(&mut self) -> Result<(), RuntimeError> {
        let iterator = self.pop()?;
        self.push(Value::Boolean(iterator.iterator_has_next()?));
        Ok(())
    }

    pub(crate) fn op_iterator_next(&mut self) -> Result<(), RuntimeError> {
        let iterator = self.pop()?;
        self.push(iterator.iterator_next()?);
        Ok(())
    }
}

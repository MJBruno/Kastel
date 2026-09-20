use super::VirtualMachine;

use crate::error::runtime_error::RuntimeError;

impl VirtualMachine {
    pub(crate) fn get_property(&mut self, wide: bool) -> Result<(), RuntimeError> {
        let constant = self.read_constant_operand(wide)?;
        let property = self.read_constant(constant)?;

        let name = property.as_string_value().ok_or(RuntimeError::TypeError)?;

        let object = self.peek()?.clone();

        self.ensure_member_access(&object, &name)?;

        let value = object.get_property(&name)?;

        self.pop()?;
        self.push(value);

        Ok(())
    }

    pub(crate) fn set_property(&mut self, wide: bool) -> Result<(), RuntimeError> {
        let constant = self.read_constant_operand(wide)?;
        let property = self.read_constant(constant)?;

        let name = property.as_string_value().ok_or(RuntimeError::TypeError)?;

        if self.stack.len() < 2 {
            return Err(RuntimeError::StackUnderflow);
        }

        let len = self.stack.len();
        let object = self.stack[len - 2].clone();
        let value = self.stack[len - 1].clone();

        self.ensure_member_access(&object, &name)?;

        object.set_property(&name, value)?;

        self.stack.truncate(len - 2);

        Ok(())
    }
}

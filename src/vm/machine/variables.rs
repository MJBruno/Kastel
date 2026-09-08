use super::VirtualMachine;
use crate::error::runtime_error::RuntimeError;

impl VirtualMachine {
    pub(crate) fn define_global(&mut self) -> Result<(), RuntimeError> {
        let constant = self.read_constant_byte()?;
        let name = constant.as_string_value().ok_or(RuntimeError::TypeError)?;
        let value = self.pop()?;
        self.globals.insert(name, value);
        Ok(())
    }

    pub(crate) fn get_global(&mut self) -> Result<(), RuntimeError> {
        let constant = self.read_constant_byte()?;
        let name = constant.as_string_value().ok_or(RuntimeError::TypeError)?;
        let value = self.globals.get(&name).cloned().ok_or(RuntimeError::TypeError)?;
        self.push(value);
        Ok(())
    }

    pub(crate) fn set_global(&mut self) -> Result<(), RuntimeError> {
        let constant = self.read_constant_byte()?;
        let name = constant.as_string_value().ok_or(RuntimeError::TypeError)?;
        if !self.globals.contains_key(&name) { return Err(RuntimeError::TypeError); }
        let value = self.peek()?.clone();
        self.globals.insert(name, value);
        Ok(())
    }

    pub(crate) fn get_local(&mut self) -> Result<(), RuntimeError> {
        let slot = self.read_byte()? as usize;
        let slot_start = self.current_frame()?.slot_start;
        let index = slot_start.checked_add(1).and_then(|i| i.checked_add(slot)).ok_or(RuntimeError::InvalidFunction)?;
        let value = self.stack.get(index).cloned().ok_or(RuntimeError::StackUnderflow)?;
        self.push(value);
        Ok(())
    }

    pub(crate) fn set_local(&mut self) -> Result<(), RuntimeError> {
        let slot = self.read_byte()? as usize;
        let value = self.peek()?.clone();
        let slot_start = self.current_frame()?.slot_start;
        let index = slot_start.checked_add(1).and_then(|i| i.checked_add(slot)).ok_or(RuntimeError::InvalidFunction)?;
        let target = self.stack.get_mut(index).ok_or(RuntimeError::StackUnderflow)?;
        *target = value;
        Ok(())
    }
}

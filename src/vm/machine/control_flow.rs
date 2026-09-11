use super::VirtualMachine;

use crate::error::runtime_error::RuntimeError;

impl VirtualMachine {
    pub(crate) fn jump(&mut self) -> Result<(), RuntimeError> {
        let offset = self.read_short()? as usize;
        self.jump_forward(offset)
    }

    pub(crate) fn jump_if_false(&mut self) -> Result<(), RuntimeError> {
        let offset = self.read_short()? as usize;

        if !self.peek()?.is_truthy() {
            self.jump_forward(offset)?;
        }

        Ok(())
    }

    pub(crate) fn jump_if_false_pop(&mut self) -> Result<(), RuntimeError> {
        let offset = self.read_short()? as usize;

        let condition = self.pop()?;

        if !condition.is_truthy() {
            self.jump_forward(offset)?;
        }

        Ok(())
    }

    pub(crate) fn loop_back(&mut self) -> Result<(), RuntimeError> {
        let offset = self.read_short()? as usize;

        let frame = self.current_frame()?;

        let new_ip = frame
            .ip
            .checked_sub(offset)
            .ok_or(RuntimeError::InvalidFunction)?;

        if new_ip >= frame.chunk.code.len() {
            return Err(RuntimeError::InvalidFunction);
        }

        self.current_frame_mut()?.ip = new_ip;

        Ok(())
    }

    fn jump_forward(&mut self, offset: usize) -> Result<(), RuntimeError> {
        let frame = self.current_frame()?;

        let new_ip = frame
            .ip
            .checked_add(offset)
            .ok_or(RuntimeError::InvalidFunction)?;

        if new_ip >= frame.chunk.code.len() {
            return Err(RuntimeError::InvalidFunction);
        }

        self.current_frame_mut()?.ip = new_ip;

        Ok(())
    }
}

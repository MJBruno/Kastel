use super::VirtualMachine;
use crate::error::runtime_error::RuntimeError;

impl VirtualMachine {
    pub(crate) fn jump(&mut self) -> Result<(), RuntimeError> {
        let offset = self.read_short()? as usize;
        let frame = self.current_frame_mut()?;
        frame.ip = frame.ip.checked_add(offset).ok_or(RuntimeError::InvalidFunction)?;
        Ok(())
    }

    pub(crate) fn jump_if_false(&mut self) -> Result<(), RuntimeError> {
        let offset = self.read_short()? as usize;
        if !self.peek()?.is_truthy() { self.jump_by(offset)?; }
        Ok(())
    }

    pub(crate) fn loop_back(&mut self) -> Result<(), RuntimeError> {
        let offset = self.read_short()? as usize;
        let frame = self.current_frame_mut()?;
        frame.ip = frame.ip.checked_sub(offset).ok_or(RuntimeError::InvalidFunction)?;
        Ok(())
    }

    fn jump_by(&mut self, offset: usize) -> Result<(), RuntimeError> {
        let frame = self.current_frame_mut()?;
        frame.ip = frame.ip.checked_add(offset).ok_or(RuntimeError::InvalidFunction)?;
        Ok(())
    }
}

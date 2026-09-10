use super::VirtualMachine;
use crate::error::runtime_error::RuntimeError;

impl VirtualMachine {
    pub(crate) fn jump(&mut self) -> Result<(), RuntimeError> {
        let offset = self.read_short()? as usize;

        let ip = self.current_frame()?.ip;

        let new_ip = ip
            .checked_add(offset)
            .ok_or(RuntimeError::InvalidFunction)?;

        self.set_ip(new_ip)
    }

    fn set_ip(&mut self, ip: usize) -> Result<(), RuntimeError> {
        let code_len = {
            let frame = self.current_frame()?;
            let closure = super::bytecode::frame_closure(&frame.closure);

            closure.function.chunk.code.len()
        };

        if ip >= code_len {
            return Err(RuntimeError::InvalidFunction);
        }

        self.current_frame_mut()?.ip = ip;

        Ok(())
    }

    pub(crate) fn jump_if_false(&mut self) -> Result<(), RuntimeError> {
        let offset = self.read_short()? as usize;

        if !self.peek()?.is_truthy() {
            self.jump_by(offset)?;
        }

        Ok(())
    }

    pub(crate) fn loop_back(&mut self) -> Result<(), RuntimeError> {
        let offset = self.read_short()? as usize;

        let ip = self.current_frame()?.ip;

        let new_ip = ip
            .checked_sub(offset)
            .ok_or(RuntimeError::InvalidFunction)?;

        self.set_ip(new_ip)
    }

    fn jump_by(&mut self, offset: usize) -> Result<(), RuntimeError> {
        let ip = self.current_frame()?.ip;

        let new_ip = ip
            .checked_add(offset)
            .ok_or(RuntimeError::InvalidFunction)?;

        self.set_ip(new_ip)
    }
}
use super::VirtualMachine;

use crate::error::runtime_error::RuntimeError;

impl VirtualMachine {
    pub(crate) fn op_push_exception_handler(
        &mut self,
    ) -> Result<(), RuntimeError> {
        let catch_raw = self.read_short()?;
        let finally_raw = self.read_short()?;

        let catch_ip = if catch_raw == u16::MAX {
            None
        } else {
            Some(catch_raw as usize)
        };

        let finally_ip = if finally_raw == u16::MAX {
            None
        } else {
            Some(finally_raw as usize)
        };

        self.register_exception_handler(catch_ip, finally_ip)
    }

    pub(crate) fn op_pop_exception_handler(
        &mut self,
    ) -> Result<(), RuntimeError> {
        self.unregister_exception_handler().map(|_| ())
    }

    pub(crate) fn op_throw(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop()?;

        Err(RuntimeError::Thrown(value))
    }

    pub(crate) fn op_finally_end(
        &mut self,
    ) -> Result<(), RuntimeError> {
        let Some(pending) = self.pending_exception.take() else {
            return Ok(());
        };

        if pending.rethrow {
            return Err(RuntimeError::Thrown(pending.value));
        }

        Ok(())
    }
}
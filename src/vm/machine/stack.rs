use super::VirtualMachine;

use crate::error::runtime_error::RuntimeError;
use crate::runtime::value::Value;

impl VirtualMachine {
    #[inline(always)]
    pub(crate) fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    #[inline(always)]
    pub(crate) fn pop(&mut self) -> Result<Value, RuntimeError> {
        match self.stack.pop() {
            Some(value) => Ok(value),

            None => Err(Self::stack_underflow()),
        }
    }

    #[inline(always)]
    pub(crate) fn peek(&self) -> Result<&Value, RuntimeError> {
        match self.stack.last() {
            Some(value) => Ok(value),

            None => Err(Self::stack_underflow()),
        }
    }

    #[cold]
    #[inline(never)]
    fn stack_underflow() -> RuntimeError {
        RuntimeError::StackUnderflow
    }
}

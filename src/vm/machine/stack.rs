use super::VirtualMachine;
use crate::error::runtime_error::RuntimeError;
use crate::runtime::value::Value;

impl VirtualMachine {
    pub(crate) fn push(&mut self, value: Value) { self.stack.push(value); }

    pub(crate) fn pop(&mut self) -> Result<Value, RuntimeError> {
        self.stack.pop().ok_or(RuntimeError::StackUnderflow)
    }

    pub(crate) fn peek(&self) -> Result<&Value, RuntimeError> {
        self.stack.last().ok_or(RuntimeError::StackUnderflow)
    }
}

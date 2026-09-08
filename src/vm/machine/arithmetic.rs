use super::VirtualMachine;
use crate::error::runtime_error::RuntimeError;
use crate::runtime::value::{ComparisonOp, NumericOp, Value};

impl VirtualMachine {
    pub(crate) fn add(&mut self) -> Result<(), RuntimeError> {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = match (a.as_string_value(), b.as_string_value()) {
            (Some(a), Some(b)) => Value::new_string(format!("{a}{b}")),
            _ => Value::binary_numeric_op(a, b, NumericOp::Add)?,
        };
        self.push(result);
        Ok(())
    }

    pub(crate) fn numeric_binary(&mut self, op: NumericOp) -> Result<(), RuntimeError> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(Value::binary_numeric_op(a, b, op)?);
        Ok(())
    }

    pub(crate) fn negate(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop()?;
        self.push(Value::negate_values(value)?);
        Ok(())
    }

    pub(crate) fn bitwise_binary(&mut self, op: u8) -> Result<(), RuntimeError> {
        let b = Self::to_bitwise_int(&self.pop()?)?;
        let a = Self::to_bitwise_int(&self.pop()?)?;
        let result = match op { 0 => a & b, 1 => a | b, 2 => a ^ b, _ => return Err(RuntimeError::InvalidFunction) };
        self.push(Value::Integer(result));
        Ok(())
    }

    pub(crate) fn bitwise_not(&mut self) -> Result<(), RuntimeError> {
        let a = Self::to_bitwise_int(&self.pop()?)?;
        self.push(Value::Integer(!a));
        Ok(())
    }

    pub(crate) fn shift(&mut self, left: bool) -> Result<(), RuntimeError> {
        let b = Self::to_bitwise_int(&self.pop()?)?;
        let a = Self::to_bitwise_int(&self.pop()?)?;
        if !(0..64).contains(&b) { return Err(RuntimeError::InvalidShiftAmount); }
        self.push(Value::Integer(if left { a << b } else { a >> b }));
        Ok(())
    }

    pub(crate) fn compare(&mut self, op: ComparisonOp) -> Result<(), RuntimeError> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(Value::compare_numeric(a, b, op)?);
        Ok(())
    }

    pub(crate) fn not(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop()?;
        self.push(Value::Boolean(!value.is_truthy()));
        Ok(())
    }

    fn to_bitwise_int(value: &Value) -> Result<i64, RuntimeError> {
        match value { Value::Integer(n) => Ok(*n), _ => Err(RuntimeError::TypeError) }
    }
}

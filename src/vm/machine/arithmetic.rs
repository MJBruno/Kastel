use super::VirtualMachine;

use crate::error::runtime_error::RuntimeError;
use crate::runtime::value::{ComparisonOp, NumericOp, Value};

impl VirtualMachine {
    #[inline]
    pub(crate) fn add(&mut self) -> Result<(), RuntimeError> {
        let b = self.pop()?;
        let a = self.pop()?;

        /*
         * Fast path entier.
         *
         * Le cas Integer + Integer est extrêmement fréquent.
         * On évite ici l'appel générique à binary_numeric_op().
         */
        if let (Value::Integer(a), Value::Integer(b)) = (&a, &b) {
            self.push(Value::Integer(a.wrapping_add(*b)));
            return Ok(());
        }

        /*
         * Concaténation des chaînes.
         */
        match (a.as_string_value(), b.as_string_value()) {
            (Some(a), Some(b)) => {
                self.push(Value::new_string(format!("{a}{b}")));
            }

            _ => {
                self.push(Value::binary_numeric_op(a, b, NumericOp::Add)?);
            }
        }

        Ok(())
    }
    #[inline]
    pub(crate) fn less_local_const(&mut self) -> Result<(), RuntimeError> {
        let slot = self.read_byte()? as usize;
        let constant_index = self.read_byte()? as usize;

        let (slot_start, _, constant) = {
            let frame = self.frames.last().ok_or(RuntimeError::InvalidFunction)?;

            if slot >= frame.local_count {
                return Err(RuntimeError::InvalidFunction);
            }

            let constant = frame
                .chunk
                .constants
                .get(constant_index)
                .cloned()
                .ok_or(RuntimeError::InvalidFunction)?;

            (frame.slot_start, frame.local_count, constant)
        };

        let local_index = slot_start
            .checked_add(1)
            .and_then(|index| index.checked_add(slot))
            .ok_or(RuntimeError::InvalidFunction)?;

        let local = self
            .stack
            .get(local_index)
            .cloned()
            .ok_or(RuntimeError::StackUnderflow)?;

        let result = Value::compare_numeric(local, constant, ComparisonOp::Less)?;

        self.push(result);

        Ok(())
    }
    #[inline]
    pub(crate) fn numeric_binary(&mut self, op: NumericOp) -> Result<(), RuntimeError> {
        let b = self.pop()?;
        let a = self.pop()?;

        /*
         * Fast path entier pour les opérations numériques
         * qui ont une représentation entière directe.
         *
         * Les opérations qui ne doivent pas être traitées ici
         * restent dans binary_numeric_op().
         */
        if let (Value::Integer(a), Value::Integer(b)) = (&a, &b) {
            let result = match op {
                NumericOp::Add => Some(a.wrapping_add(*b)),

                NumericOp::Subtract => Some(a.wrapping_sub(*b)),

                NumericOp::Multiply => Some(a.wrapping_mul(*b)),

                _ => None,
            };

            if let Some(result) = result {
                self.push(Value::Integer(result));
                return Ok(());
            }
        }

        self.push(Value::binary_numeric_op(a, b, op)?);

        Ok(())
    }

    #[inline]
    pub(crate) fn negate(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop()?;

        match value {
            Value::Integer(value) => {
                self.push(Value::Integer(value.wrapping_neg()));
            }

            value => {
                self.push(Value::negate_values(value)?);
            }
        }

        Ok(())
    }

    #[inline]
    pub(crate) fn bitwise_binary(&mut self, op: u8) -> Result<(), RuntimeError> {
        let b = Self::to_bitwise_int(&self.pop()?)?;
        let a = Self::to_bitwise_int(&self.pop()?)?;

        let result = match op {
            0 => a & b,
            1 => a | b,
            2 => a ^ b,
            _ => {
                return Err(RuntimeError::InvalidFunction);
            }
        };

        self.push(Value::Integer(result));

        Ok(())
    }

    #[inline]
    pub(crate) fn bitwise_not(&mut self) -> Result<(), RuntimeError> {
        let a = Self::to_bitwise_int(&self.pop()?)?;

        self.push(Value::Integer(!a));

        Ok(())
    }

    #[inline]
    pub(crate) fn shift(&mut self, left: bool) -> Result<(), RuntimeError> {
        let b = Self::to_bitwise_int(&self.pop()?)?;
        let a = Self::to_bitwise_int(&self.pop()?)?;

        if !(0..64).contains(&b) {
            return Err(RuntimeError::InvalidShiftAmount);
        }

        self.push(Value::Integer(if left { a << b } else { a >> b }));

        Ok(())
    }

    #[inline]
    pub(crate) fn compare(&mut self, op: ComparisonOp) -> Result<(), RuntimeError> {
        let b = self.pop()?;
        let a = self.pop()?;

        /*
         * Fast path entier.
         *
         * Cela couvre les comparaisons classiques :
         *
         *     i < n
         *     i > n
         *
         * lorsque le compilateur n'a pas utilisé une
         * instruction spécialisée.
         */
        if let (Value::Integer(a), Value::Integer(b)) = (&a, &b) {
            let result = match op {
                ComparisonOp::Equal => *a == *b,
                ComparisonOp::Greater => *a > *b,
                ComparisonOp::Less => *a < *b,
            };

            self.push(Value::Boolean(result));

            return Ok(());
        }

        self.push(Value::compare_numeric(a, b, op)?);

        Ok(())
    }

    #[inline]
    pub(crate) fn not(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop()?;

        self.push(Value::Boolean(!value.is_truthy()));

        Ok(())
    }

    #[inline]
    fn to_bitwise_int(value: &Value) -> Result<i64, RuntimeError> {
        match value {
            Value::Integer(n) => Ok(*n),

            _ => Err(RuntimeError::TypeError),
        }
    }
}

use super::VirtualMachine;

use crate::error::runtime_error::RuntimeError;
use crate::runtime::value::{NumericOp, Value};

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

        let value = self
            .globals
            .get(&name)
            .cloned()
            .ok_or(RuntimeError::TypeError)?;

        self.push(value);

        Ok(())
    }

    pub(crate) fn set_global(&mut self) -> Result<(), RuntimeError> {
        let constant = self.read_constant_byte()?;
        let name = constant.as_string_value().ok_or(RuntimeError::TypeError)?;

        if !self.globals.contains_key(&name) {
            return Err(RuntimeError::TypeError);
        }

        let value = self.peek()?.clone();
        self.globals.insert(name, value);

        Ok(())
    }

    pub(crate) fn get_local(&mut self) -> Result<(), RuntimeError> {
        let slot = self.read_byte()? as usize;

        let (slot_start, local_count) = {
            let frame = self.frames.last().ok_or(RuntimeError::InvalidFunction)?;

            (frame.slot_start, frame.local_count)
        };

        if slot >= local_count {
            return Err(RuntimeError::InvalidFunction);
        }

        let index = slot_start
            .checked_add(1)
            .and_then(|index| index.checked_add(slot))
            .ok_or(RuntimeError::InvalidFunction)?;

        let value = self
            .stack
            .get(index)
            .cloned()
            .ok_or(RuntimeError::StackUnderflow)?;

        self.stack.push(value);

        Ok(())
    }

    pub(crate) fn set_local(&mut self) -> Result<(), RuntimeError> {
        let slot = self.read_byte()? as usize;

        let value = self
            .stack
            .last()
            .cloned()
            .ok_or(RuntimeError::StackUnderflow)?;

        let (slot_start, local_count) = {
            let frame = self.frames.last().ok_or(RuntimeError::InvalidFunction)?;

            (frame.slot_start, frame.local_count)
        };

        if slot >= local_count {
            return Err(RuntimeError::InvalidFunction);
        }

        let index = slot_start
            .checked_add(1)
            .and_then(|index| index.checked_add(slot))
            .ok_or(RuntimeError::InvalidFunction)?;

        let target = self
            .stack
            .get_mut(index)
            .ok_or(RuntimeError::StackUnderflow)?;

        *target = value;

        Ok(())
    }

    #[inline(always)]
    pub(crate) fn set_local_pop(&mut self) -> Result<(), RuntimeError> {
        let slot = self.read_byte()? as usize;

        let value = self.pop()?;

        let (slot_start, local_count) = {
            let frame = self.frames.last().ok_or(RuntimeError::InvalidFunction)?;

            (frame.slot_start, frame.local_count)
        };

        if slot >= local_count {
            return Err(RuntimeError::InvalidFunction);
        }

        let index = slot_start
            .checked_add(1)
            .and_then(|index| index.checked_add(slot))
            .ok_or(RuntimeError::InvalidFunction)?;

        let target = self
            .stack
            .get_mut(index)
            .ok_or(RuntimeError::StackUnderflow)?;

        *target = value;

        Ok(())
    }

    #[inline(always)]
    pub(crate) fn add_local_const(&mut self) -> Result<(), RuntimeError> {
        let slot = self.read_byte()? as usize;
        let constant_index = self.read_byte()? as usize;

        let (slot_start, constant) = {
            let frame = self.frames.last().ok_or(RuntimeError::InvalidFunction)?;

            if slot >= frame.local_count {
                return Err(RuntimeError::InvalidFunction);
            }

            let constant = frame
                .chunk
                .constants
                .get(constant_index)
                .ok_or(RuntimeError::InvalidFunction)?;

            (frame.slot_start, constant)
        };

        let local_index = slot_start
            .checked_add(1)
            .and_then(|index| index.checked_add(slot))
            .ok_or(RuntimeError::InvalidFunction)?;

        /*
         * Hot path :
         *
         *     Integer + Integer
         *
         * Aucun clone de la valeur locale et aucun appel au helper
         * générique.
         */
        if let (Some(Value::Integer(local)), Value::Integer(constant)) =
            (self.stack.get(local_index), constant)
        {
            let result = local.wrapping_add(*constant);

            let target = self
                .stack
                .get_mut(local_index)
                .ok_or(RuntimeError::StackUnderflow)?;

            *target = Value::Integer(result);

            return Ok(());
        }

        /*
         * Fallback pour les autres combinaisons numériques.
         */
        let local = self
            .stack
            .get(local_index)
            .cloned()
            .ok_or(RuntimeError::StackUnderflow)?;

        let result = Value::binary_numeric_op(local, constant.clone(), NumericOp::Add)?;

        let target = self
            .stack
            .get_mut(local_index)
            .ok_or(RuntimeError::StackUnderflow)?;

        *target = result;

        Ok(())
    }
}

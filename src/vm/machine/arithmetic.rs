use super::VirtualMachine;

use crate::error::runtime_error::RuntimeError;
use crate::runtime::value::{ComparisonOp, NumericOp, Value};

impl VirtualMachine {
    #[inline]
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

    #[inline]
    pub(crate) fn numeric_binary(&mut self, op: NumericOp) -> Result<(), RuntimeError> {
        let b = self.pop()?;
        let a = self.pop()?;

        self.push(Value::binary_numeric_op(a, b, op)?);

        Ok(())
    }

    #[inline]
    pub(crate) fn negate(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop()?;

        self.push(Value::negate_values(value)?);

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

        self.push(Value::compare_numeric(a, b, op)?);

        Ok(())
    }

    #[inline(always)]
    pub(crate) fn less_local_const(&mut self) -> Result<(), RuntimeError> {
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
         *     Integer < Integer
         *
         * On ne clone plus le local.
         * La comparaison s'effectue directement sur la référence
         * contenue dans la stack.
         */
        if let (Some(Value::Integer(local)), Value::Integer(constant)) =
            (self.stack.get(local_index), constant)
        {
            self.push(Value::Boolean(*local < *constant));

            return Ok(());
        }

        /*
         * Fallback uniquement pour les autres types numériques.
         *
         * Ici le clone est nécessaire pour transmettre les Values
         * au chemin générique.
         */
        let local = self
            .stack
            .get(local_index)
            .cloned()
            .ok_or(RuntimeError::StackUnderflow)?;

        let constant = constant.clone();

        let result = Value::compare_numeric(local, constant, ComparisonOp::Less)?;

        self.push(result);

        Ok(())
    }
    #[inline(always)]
    pub(crate) fn less_local_const_jump(&mut self) -> Result<(), RuntimeError> {
        let slot = self.read_byte()? as usize;
        let constant_index = self.read_byte()? as usize;
        let offset = self.read_short()? as usize;

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

        let is_less = match (self.stack.get(local_index), constant) {
            (Some(Value::Integer(local)), Value::Integer(constant)) => *local < *constant,

            (Some(local), constant) => {
                match Value::compare_numeric(local.clone(), constant.clone(), ComparisonOp::Less)? {
                    Value::Boolean(value) => value,
                    _ => return Err(RuntimeError::InvalidFunction),
                }
            }

            (None, _) => {
                return Err(RuntimeError::StackUnderflow);
            }
        };

        if !is_less {
            let frame = self.frames.last().ok_or(RuntimeError::InvalidFunction)?;

            let new_ip = frame
                .ip
                .checked_add(offset)
                .ok_or(RuntimeError::InvalidFunction)?;

            if new_ip >= frame.chunk.code.len() {
                return Err(RuntimeError::InvalidFunction);
            }

            self.frames
                .last_mut()
                .ok_or(RuntimeError::InvalidFunction)?
                .ip = new_ip;
        }

        Ok(())
    }

    #[inline(always)]
    pub(crate) fn loop_less_add_local_const(&mut self) -> Result<(), RuntimeError> {
        let instruction_start = {
            let frame = self.frames.last().ok_or(RuntimeError::InvalidFunction)?;

            frame
                .ip
                .checked_sub(1)
                .ok_or(RuntimeError::InvalidFunction)?
        };

        // ============================================================
        // CACHE
        // ============================================================

        if let Some(cache) = self
            .frames
            .last()
            .and_then(|frame| frame.hot_loop_cache.clone())
        {
            match cache {
                crate::vm::machine::HotLoopCache::Integer {
                    instruction_start: cached_start,
                    local_index,
                    limit,
                    increment,
                } if cached_start == instruction_start => {
                    let target = self
                        .stack
                        .get_mut(local_index)
                        .ok_or(RuntimeError::StackUnderflow)?;

                    match target {
                        Value::Integer(local) => {
                            if *local < limit {
                                *local = local.wrapping_add(increment);

                                self.frames
                                    .last_mut()
                                    .ok_or(RuntimeError::InvalidFunction)?
                                    .ip = instruction_start;

                                return Ok(());
                            }

                            self.frames
                                .last_mut()
                                .ok_or(RuntimeError::InvalidFunction)?
                                .ip = instruction_start + 4;

                            return Ok(());
                        }

                        _ => return Err(RuntimeError::InvalidFunction),
                    }
                }

                crate::vm::machine::HotLoopCache::Float {
                    instruction_start: cached_start,
                    local_index,
                    limit,
                    increment,
                } if cached_start == instruction_start => {
                    let target = self
                        .stack
                        .get_mut(local_index)
                        .ok_or(RuntimeError::StackUnderflow)?;

                    match target {
                        Value::Float(local) => {
                            if *local < limit {
                                *local += increment;

                                self.frames
                                    .last_mut()
                                    .ok_or(RuntimeError::InvalidFunction)?
                                    .ip = instruction_start;

                                return Ok(());
                            }

                            self.frames
                                .last_mut()
                                .ok_or(RuntimeError::InvalidFunction)?
                                .ip = instruction_start + 4;

                            return Ok(());
                        }

                        _ => return Err(RuntimeError::InvalidFunction),
                    }
                }

                _ => {}
            }
        }

        // ============================================================
        // PREMIER PASSAGE : DECODE
        // ============================================================

        let slot = self.read_byte()? as usize;
        let limit_index = self.read_byte()? as usize;
        let increment_index = self.read_byte()? as usize;

        let (slot_start, limit, increment) = {
            let frame = self.frames.last().ok_or(RuntimeError::InvalidFunction)?;

            if slot >= frame.local_count {
                return Err(RuntimeError::InvalidFunction);
            }

            let limit = frame
                .chunk
                .constants
                .get(limit_index)
                .cloned()
                .ok_or(RuntimeError::InvalidFunction)?;

            let increment = frame
                .chunk
                .constants
                .get(increment_index)
                .cloned()
                .ok_or(RuntimeError::InvalidFunction)?;

            (frame.slot_start, limit, increment)
        };

        let local_index = slot_start
            .checked_add(1)
            .and_then(|index| index.checked_add(slot))
            .ok_or(RuntimeError::InvalidFunction)?;

        // ============================================================
        // INTEGER LOOP ELIMINATION
        //
        // while i < limit {
        //     i += increment
        // }
        //
        // devient directement :
        //
        // i = i + ceil((limit - i) / increment)
        //
        // uniquement quand increment > 0 et que le résultat tient
        // dans i64.
        // ============================================================

        if let (Value::Integer(limit), Value::Integer(increment)) = (&limit, &increment) {
            let target = self
                .stack
                .get_mut(local_index)
                .ok_or(RuntimeError::StackUnderflow)?;

            let local = match target {
                Value::Integer(value) => *value,
                _ => return Err(RuntimeError::InvalidFunction),
            };

            if local >= *limit {
                self.frames
                    .last_mut()
                    .ok_or(RuntimeError::InvalidFunction)?
                    .ip = instruction_start + 4;

                return Ok(());
            }

            if *increment > 0 {
                let local128 = local as i128;
                let limit128 = *limit as i128;
                let increment128 = *increment as i128;

                let distance = limit128 - local128;
                let steps = (distance + increment128 - 1) / increment128;
                let final_value = local128 + steps * increment128;

                if let Ok(final_value) = i64::try_from(final_value) {
                    *target = Value::Integer(final_value);

                    self.frames
                        .last_mut()
                        .ok_or(RuntimeError::InvalidFunction)?
                        .ip = instruction_start + 4;

                    return Ok(());
                }
            }

            // Cas overflow / increment non positif :
            // retour au comportement itératif normal.
            self.frames
                .last_mut()
                .ok_or(RuntimeError::InvalidFunction)?
                .hot_loop_cache = Some(crate::vm::machine::HotLoopCache::Integer {
                instruction_start,
                local_index,
                limit: *limit,
                increment: *increment,
            });

            *target = Value::Integer(local.wrapping_add(*increment));

            self.frames
                .last_mut()
                .ok_or(RuntimeError::InvalidFunction)?
                .ip = instruction_start;

            return Ok(());
        }

        // ============================================================
        // FLOAT
        // ============================================================

        if let (Value::Float(limit), Value::Float(increment)) = (&limit, &increment) {
            self.frames
                .last_mut()
                .ok_or(RuntimeError::InvalidFunction)?
                .hot_loop_cache = Some(crate::vm::machine::HotLoopCache::Float {
                instruction_start,
                local_index,
                limit: *limit,
                increment: *increment,
            });

            let target = self
                .stack
                .get_mut(local_index)
                .ok_or(RuntimeError::StackUnderflow)?;

            match target {
                Value::Float(local) => {
                    if *local < *limit {
                        *local += *increment;

                        self.frames
                            .last_mut()
                            .ok_or(RuntimeError::InvalidFunction)?
                            .ip = instruction_start;
                    } else {
                        self.frames
                            .last_mut()
                            .ok_or(RuntimeError::InvalidFunction)?
                            .ip = instruction_start + 4;
                    }

                    return Ok(());
                }

                _ => return Err(RuntimeError::InvalidFunction),
            }
        }

        // ============================================================
        // FALLBACK GENERIC
        // ============================================================

        let target = self
            .stack
            .get_mut(local_index)
            .ok_or(RuntimeError::StackUnderflow)?;

        let condition = Value::compare_numeric(target.clone(), limit.clone(), ComparisonOp::Less)?;

        let is_less = match condition {
            Value::Boolean(value) => value,
            _ => return Err(RuntimeError::InvalidFunction),
        };

        if !is_less {
            self.frames
                .last_mut()
                .ok_or(RuntimeError::InvalidFunction)?
                .ip = instruction_start + 4;

            return Ok(());
        }

        let result = Value::binary_numeric_op(target.clone(), increment, NumericOp::Add)?;

        *target = result;

        self.frames
            .last_mut()
            .ok_or(RuntimeError::InvalidFunction)?
            .ip = instruction_start;

        Ok(())
    }

    #[inline(always)]
    pub(crate) fn not(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop()?;

        self.push(Value::Boolean(!value.is_truthy()));

        Ok(())
    }

    #[inline(always)]
    fn to_bitwise_int(value: &Value) -> Result<i64, RuntimeError> {
        match value {
            Value::Integer(n) => Ok(*n),

            _ => Err(RuntimeError::TypeError),
        }
    }
}

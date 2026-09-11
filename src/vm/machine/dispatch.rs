use super::VirtualMachine;

use crate::{
    error::runtime_error::RuntimeError,
    runtime::{
        object::Object,
        value::{ComparisonOp, NumericOp, Value},
    },
};

impl VirtualMachine {
    #[inline(always)]
    pub(crate) fn dispatch(
        &mut self,
        instruction: u8,
    ) -> Result<bool, RuntimeError> {
        #[cfg(feature = "profile")]
        self.profile_instruction(instruction);

        match instruction {
            // ========================================================
            // CONSTANTS / GLOBALS
            // ========================================================

            0 => {
                let constant = self.read_constant_byte()?;
                self.push(constant);
            }

            21 => {
                self.define_global()?;
            }

            22 => {
                self.set_global()?;
            }

            23 => {
                self.get_global()?;
            }

            24 => {
                self.set_local()?;
            }

            25 => {
                self.get_local()?;
            }

            62 => {
                self.set_local_pop()?;
            }

            63 => {
                self.add_local_const()?;
            }

            66 => {
                self.less_local_const_jump()?;
            }

            67 => {
                self.loop_less_add_local_const()?;
            }

            47 => {
                let index = self.read_byte()? as usize;
                self.get_upvalue(index)?;
            }

            48 => {
                let index = self.read_byte()? as usize;
                self.set_upvalue(index)?;
            }

            // ========================================================
            // LITERALS
            // ========================================================

            1 => {
                self.push(Value::Nil);
            }

            2 => {
                self.push(Value::Boolean(true));
            }

            3 => {
                self.push(Value::Boolean(false));
            }

            // ========================================================
            // ARITHMETIC
            // ========================================================

            9 => {
                self.add()?;
            }

            10 => {
                self.numeric_binary(NumericOp::Subtract)?;
            }

            11 => {
                self.numeric_binary(NumericOp::Multiply)?;
            }

            12 => {
                self.numeric_binary(NumericOp::Divide)?;
            }

            13 => {
                self.numeric_binary(NumericOp::Modulo)?;
            }

            14 => {
                self.negate()?;
            }

            // ========================================================
            // BITWISE
            // ========================================================

            15 => {
                self.bitwise_binary(0)?;
            }

            16 => {
                self.bitwise_binary(1)?;
            }

            17 => {
                self.bitwise_binary(2)?;
            }

            18 => {
                self.bitwise_not()?;
            }

            19 => {
                self.shift(true)?;
            }

            20 => {
                self.shift(false)?;
            }

            // ========================================================
            // COMPARISON
            // ========================================================

            4 => {
                let b = self.pop()?;
                let a = self.pop()?;

                self.push(Value::Boolean(Value::equals(a, b)));
            }

            5 => {
                self.compare(ComparisonOp::Greater)?;
            }

            6 => {
                self.compare(ComparisonOp::Less)?;
            }

            64 => {
                self.less_local_const()?;
            }

            7 => {
                self.not()?;
            }

            8 => {
                let right = self.pop()?;
                let left = self.pop()?;

                let result = Self::is_value_instance_of(&left, &right)?;

                self.push(Value::Boolean(result));
            }

            // ========================================================
            // MODULES
            // ========================================================

            26 => {
                self.import_module()?;
            }

            // ========================================================
            // CONTROL FLOW
            // ========================================================

            27 => {
                self.jump_if_false()?;
            }

            28 => {
                self.jump()?;
            }

            29 => {
                self.pop()?;
            }

            30 => {
                self.loop_back()?;
            }

            65 => {
                self.jump_if_false_pop()?;
            }

            // ========================================================
            // CALL / FUNCTIONS
            // ========================================================

            31 => {
                let arg_count = self.read_byte()? as usize;

                self.execute_call(arg_count)?;
            }

            46 => {
                self.op_closure()?;
            }

            // ========================================================
            // ARRAYS
            // ========================================================

            32 => {
                let count = self.read_byte()? as usize;

                self.op_array(count)?;
            }

            33 => {
                if let Some(Value::Object(handle)) =
                    self.stack.get(self.stack.len().saturating_sub(2))
                {
                    let object = handle.borrow();

                    match &*object {
                        Object::Array(_) => {
                            drop(object);
                            self.op_get_array_index()?;
                        }

                        Object::Dict(_) => {
                            drop(object);
                            self.op_get_dict_index()?;
                        }

                        _ => {
                            return Err(RuntimeError::NotIndexable);
                        }
                    }
                } else {
                    return Err(RuntimeError::NotIndexable);
                }
            }

            34 => {
                if let Some(Value::Object(handle)) =
                    self.stack.get(self.stack.len().saturating_sub(3))
                {
                    let object = handle.borrow();

                    match &*object {
                        Object::Array(_) => {
                            drop(object);
                            self.op_set_array_index()?;
                        }

                        Object::Dict(_) => {
                            drop(object);
                            self.op_set_dict_index()?;
                        }

                        _ => {
                            return Err(RuntimeError::NotIndexable);
                        }
                    }
                } else {
                    return Err(RuntimeError::NotIndexable);
                }
            }

            35 => {
                self.op_array_length()?;
            }

            36 => {
                self.op_array_push()?;
            }

            37 => {
                self.op_array_pop()?;
            }

            38 => {
                self.op_array_insert()?;
            }

            39 => {
                self.op_array_remove()?;
            }

            40 => {
                self.op_array_clear()?;
            }

            41 => {
                self.op_array_contains()?;
            }

            // ========================================================
            // OBJECT
            // ========================================================

            42 => {
                let pair_count = self.read_byte()? as usize;

                self.op_object(pair_count)?;
            }

            // ========================================================
            // ITERATORS
            // ========================================================

            43 => {
                self.op_get_iterator()?;
            }

            44 => {
                self.op_iterator_has_next()?;
            }

            45 => {
                self.op_iterator_next()?;
            }

            // ========================================================
            // PROPERTIES
            // ========================================================

            51 => {
                self.get_property()?;
            }

            52 => {
                self.set_property()?;
            }

            // ========================================================
            // METHOD CALLS
            // ========================================================

            53 => {
                let method_constant = self.read_byte()? as usize;
                let arg_count = self.read_byte()? as usize;

                self.op_invoke_method(
                    method_constant,
                    arg_count,
                )?;
            }

            54 => {
                let method_constant = self.read_byte()? as usize;
                let arg_count = self.read_byte()? as usize;

                self.op_invoke_base_method(
                    method_constant,
                    arg_count,
                )?;
            }

            // ========================================================
            // CLASSES / INTERFACES
            // ========================================================

            55 => {
                let base_count = self.read_byte()? as usize;
                let method_count = self.read_byte()? as usize;

                self.op_class(
                    base_count,
                    method_count,
                )?;
            }

            56 => {
                let arg_count = self.read_byte()? as usize;

                self.op_new_instance(arg_count)?;
            }

            57 => {
                let base_count = self.read_byte()? as usize;
                let method_count = self.read_byte()? as usize;

                self.op_interface(
                    base_count,
                    method_count,
                )?;
            }

            // ========================================================
            // RETURN / HALT
            // ========================================================

            49 => {
                self.execute_return()?;

                return Ok(self.frames.is_empty());
            }

            50 => {
                return Ok(true);
            }

            // ========================================================
            // EXCEPTIONS
            // ========================================================

            58 => {
                self.op_push_exception_handler()?;
            }

            59 => {
                self.op_pop_exception_handler()?;
            }

            60 => {
                self.op_throw()?;
            }

            61 => {
                self.op_finally_end()?;
            }

            // ========================================================
            // INVALID
            // ========================================================

            _ => {
                return Err(RuntimeError::InvalidOpcode(instruction));
            }
        }

        Ok(false)
    }
}
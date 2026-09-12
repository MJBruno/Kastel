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
        let result = match instruction {
            // ========================================================
            // CONSTANTS / GLOBALS
            // ========================================================
            0 => {
                let constant = self.read_constant_byte()?;
                self.push(constant);
                Ok(false)
            }

            21 => {
                self.define_global()?;
                Ok(false)
            }

            22 => {
                self.set_global()?;
                Ok(false)
            }

            23 => {
                self.get_global()?;
                Ok(false)
            }

            24 => {
                self.set_local()?;
                Ok(false)
            }

            25 => {
                self.get_local()?;
                Ok(false)
            }

            62 => {
                self.set_local_pop()?;
                Ok(false)
            }

            63 => {
                self.add_local_const()?;
                Ok(false)
            }

            68 => {
                self.add_local_local()?;
                Ok(false)
            }

            66 => {
                self.less_local_const_jump()?;
                Ok(false)
            }

            67 => {
                self.loop_less_add_local_const()?;
                Ok(false)
            }

            47 => {
                let index = self.read_byte()? as usize;
                self.get_upvalue(index)?;
                Ok(false)
            }

            48 => {
                let index = self.read_byte()? as usize;
                self.set_upvalue(index)?;
                Ok(false)
            }

            // ========================================================
            // LITERALS
            // ========================================================
            1 => {
                self.push(Value::Nil);
                Ok(false)
            }

            2 => {
                self.push(Value::Boolean(true));
                Ok(false)
            }

            3 => {
                self.push(Value::Boolean(false));
                Ok(false)
            }

            // ========================================================
            // ARITHMETIC
            // ========================================================
            9 => {
                self.add()?;
                Ok(false)
            }

            10 => {
                self.numeric_binary(NumericOp::Subtract)?;
                Ok(false)
            }

            11 => {
                self.numeric_binary(NumericOp::Multiply)?;
                Ok(false)
            }

            12 => {
                self.numeric_binary(NumericOp::Divide)?;
                Ok(false)
            }

            13 => {
                self.numeric_binary(NumericOp::Modulo)?;
                Ok(false)
            }

            14 => {
                self.negate()?;
                Ok(false)
            }

            // ========================================================
            // BITWISE
            // ========================================================
            15 => {
                self.bitwise_binary(0)?;
                Ok(false)
            }

            16 => {
                self.bitwise_binary(1)?;
                Ok(false)
            }

            17 => {
                self.bitwise_binary(2)?;
                Ok(false)
            }

            18 => {
                self.bitwise_not()?;
                Ok(false)
            }

            19 => {
                self.shift(true)?;
                Ok(false)
            }

            20 => {
                self.shift(false)?;
                Ok(false)
            }

            // ========================================================
            // COMPARISON
            // ========================================================
            4 => {
                let b = self.pop()?;
                let a = self.pop()?;

                self.push(Value::Boolean(Value::equals(a, b)));
                Ok(false)
            }

            5 => {
                self.compare(ComparisonOp::Greater)?;
                Ok(false)
            }

            6 => {
                self.compare(ComparisonOp::Less)?;
                Ok(false)
            }

            64 => {
                self.less_local_const()?;
                Ok(false)
            }

            7 => {
                self.not()?;
                Ok(false)
            }

            8 => {
                let right = self.pop()?;
                let left = self.pop()?;

                let result = Self::is_value_instance_of(&left, &right)?;

                self.push(Value::Boolean(result));
                Ok(false)
            }

            // ========================================================
            // MODULES
            // ========================================================
            26 => {
                self.import_module()?;
                Ok(false)
            }

            // ========================================================
            // CONTROL FLOW
            // ========================================================
            27 => {
                self.jump_if_false()?;
                Ok(false)
            }

            28 => {
                self.jump()?;
                Ok(false)
            }

            29 => {
                self.pop()?;
                Ok(false)
            }

            65 => {
                self.jump_if_false_pop()?;
                Ok(false)
            }

            // ========================================================
            // CALL / FUNCTIONS
            // ========================================================
            31 => {
                let arg_count = self.read_byte()? as usize;

                self.execute_call(arg_count)?;
                Ok(false)
            }

            46 => {
                self.op_closure()?;
                Ok(false)
            }

            // ========================================================
            // ARRAYS
            // ========================================================
            32 => {
                let count = self.read_byte()? as usize;

                self.op_array(count)?;
                Ok(false)
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

                Ok(false)
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

                Ok(false)
            }

            35 => {
                self.op_array_length()?;
                Ok(false)
            }

            36 => {
                self.op_array_push()?;
                Ok(false)
            }

            37 => {
                self.op_array_pop()?;
                Ok(false)
            }

            38 => {
                self.op_array_insert()?;
                Ok(false)
            }

            39 => {
                self.op_array_remove()?;
                Ok(false)
            }

            40 => {
                self.op_array_clear()?;
                Ok(false)
            }

            41 => {
                self.op_array_contains()?;
                Ok(false)
            }

            // ========================================================
            // OBJECT
            // ========================================================
            42 => {
                let pair_count = self.read_byte()? as usize;

                self.op_object(pair_count)?;
                Ok(false)
            }

            // ========================================================
            // ITERATORS
            // ========================================================
            43 => {
                self.op_get_iterator()?;
                Ok(false)
            }

            44 => {
                self.op_iterator_has_next()?;
                Ok(false)
            }

            45 => {
                self.op_iterator_next()?;
                Ok(false)
            }

            // ========================================================
            // PROPERTIES
            // ========================================================
            51 => {
                self.get_property()?;
                Ok(false)
            }

            52 => {
                self.set_property()?;
                Ok(false)
            }

            // ========================================================
            // METHOD CALLS
            // ========================================================
            53 => {
                let method_constant = self.read_byte()? as usize;
                let arg_count = self.read_byte()? as usize;

                self.op_invoke_method(method_constant, arg_count)?;
                Ok(false)
            }

            54 => {
                let method_constant = self.read_byte()? as usize;
                let arg_count = self.read_byte()? as usize;

                self.op_invoke_base_method(method_constant, arg_count)?;
                Ok(false)
            }

            // ========================================================
            // CLASSES / INTERFACES
            // ========================================================
            55 => {
                let base_count = self.read_byte()? as usize;
                let method_count = self.read_byte()? as usize;

                self.op_class(base_count, method_count)?;
                Ok(false)
            }

            56 => {
                let arg_count = self.read_byte()? as usize;

                self.op_new_instance(arg_count)?;
                Ok(false)
            }

            57 => {
                let base_count = self.read_byte()? as usize;
                let method_count = self.read_byte()? as usize;

                self.op_interface(base_count, method_count)?;
                Ok(false)
            }

            // ========================================================
            // RETURN / HALT
            // ========================================================
            49 => {
                self.execute_return()?;
                Ok(self.frames.is_empty())
            }

            50 => {
                Ok(true)
            }

            // ========================================================
            // EXCEPTIONS
            // ========================================================
            58 => {
                self.op_push_exception_handler()?;
                Ok(false)
            }

            59 => {
                self.op_pop_exception_handler()?;
                Ok(false)
            }

            60 => {
                self.op_throw()?;
                Ok(false)
            }

            61 => {
                self.op_finally_end()?;
                Ok(false)
            }

            // ========================================================
            // INVALID
            // ========================================================
            _ => Err(RuntimeError::InvalidOpcode(instruction)),
        };

        result
    }
}
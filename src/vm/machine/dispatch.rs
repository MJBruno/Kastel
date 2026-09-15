use super::VirtualMachine;

use crate::{
    bytecode::chunk::OpCode,
    error::runtime_error::RuntimeError,
    runtime::{
        object::Object,
        value::{ComparisonOp, NumericOp, Value},
    },
};

impl VirtualMachine {
    #[inline(always)]
    pub(crate) fn dispatch(&mut self, instruction: u8) -> Result<bool, RuntimeError> {
        let opcode =
            OpCode::from_byte(instruction).map_err(|_| RuntimeError::InvalidOpcode(instruction))?;

        match opcode {
            // ========================================================
            // CONSTANTS
            // ========================================================
            OpCode::Constant => {
                let constant = self.read_constant_byte()?;
                self.push(constant);
                Ok(false)
            }

            OpCode::None => {
                self.push(Value::None);
                Ok(false)
            }

            OpCode::True => {
                self.push(Value::Boolean(true));
                Ok(false)
            }

            OpCode::False => {
                self.push(Value::Boolean(false));
                Ok(false)
            }

            // ========================================================
            // COMPARISON / LOGICAL
            // ========================================================
            OpCode::Equal => {
                let b = self.pop()?;
                let a = self.pop()?;

                self.push(Value::Boolean(Value::equals(a, b)));
                Ok(false)
            }

            OpCode::Greater => {
                self.compare(ComparisonOp::Greater)?;
                Ok(false)
            }

            OpCode::Less => {
                self.compare(ComparisonOp::Less)?;
                Ok(false)
            }

            OpCode::Not => {
                self.not()?;
                Ok(false)
            }

            OpCode::Is => {
                let right = self.pop()?;
                let left = self.pop()?;

                let result = Self::is_value_instance_of(&left, &right)?;

                self.push(Value::Boolean(result));
                Ok(false)
            }

            // ========================================================
            // ARITHMETIC
            // ========================================================
            OpCode::Add => {
                self.add()?;
                Ok(false)
            }

            OpCode::Subtract => {
                self.numeric_binary(NumericOp::Subtract)?;
                Ok(false)
            }

            OpCode::Multiply => {
                self.numeric_binary(NumericOp::Multiply)?;
                Ok(false)
            }

            OpCode::Divide => {
                self.numeric_binary(NumericOp::Divide)?;
                Ok(false)
            }

            OpCode::Modulo => {
                self.numeric_binary(NumericOp::Modulo)?;
                Ok(false)
            }

            OpCode::Negate => {
                self.negate()?;
                Ok(false)
            }

            // ========================================================
            // BITWISE
            // ========================================================
            OpCode::BitAnd => {
                self.bitwise_binary(0)?;
                Ok(false)
            }

            OpCode::BitOr => {
                self.bitwise_binary(1)?;
                Ok(false)
            }

            OpCode::BitXor => {
                self.bitwise_binary(2)?;
                Ok(false)
            }

            OpCode::BitNot => {
                self.bitwise_not()?;
                Ok(false)
            }

            OpCode::ShiftLeft => {
                self.shift(true)?;
                Ok(false)
            }

            OpCode::ShiftRight => {
                self.shift(false)?;
                Ok(false)
            }

            // ========================================================
            // GLOBALS / LOCALS
            // ========================================================
            OpCode::DefineGlobal => {
                self.define_global()?;
                Ok(false)
            }

            OpCode::SetGlobal => {
                self.set_global()?;
                Ok(false)
            }

            OpCode::GetGlobal => {
                self.get_global()?;
                Ok(false)
            }

            OpCode::SetLocal => {
                self.set_local()?;
                Ok(false)
            }

            OpCode::GetLocal => {
                self.get_local()?;
                Ok(false)
            }

            // ========================================================
            // MODULES
            // ========================================================
            OpCode::Import => {
                self.import_module()?;
                Ok(false)
            }

            OpCode::ImportAll => {
                self.import_all()?;
                Ok(false)
            }

            // ========================================================
            // CONTROL FLOW
            // ========================================================
            OpCode::JumpIfFalse => {
                self.jump_if_false()?;
                Ok(false)
            }

            OpCode::Jump => {
                self.jump()?;
                Ok(false)
            }

            OpCode::Pop => {
                self.pop()?;
                Ok(false)
            }

            OpCode::Loop => {
                self.loop_back()?;
                Ok(false)
            }

            // ========================================================
            // CALL / FUNCTIONS
            // ========================================================
            OpCode::Call => {
                let arg_count = self.read_byte()? as usize;

                self.execute_call(arg_count)?;
                Ok(false)
            }

            OpCode::Closure => {
                self.op_closure()?;
                Ok(false)
            }

            OpCode::GetUpvalue => {
                let index = self.read_byte()? as usize;

                self.get_upvalue(index)?;
                Ok(false)
            }

            OpCode::SetUpvalue => {
                let index = self.read_byte()? as usize;

                self.set_upvalue(index)?;
                Ok(false)
            }

            // ========================================================
            // ARRAYS
            // ========================================================
            OpCode::Array => {
                let count = self.read_byte()? as usize;

                self.op_array(count)?;
                Ok(false)
            }

            OpCode::Tuple => {
                let count = self.read_byte()? as usize;

                self.op_tuple(count)?;
                Ok(false)
            }

            OpCode::GetIndex => {
                if let Some(Value::Object(handle)) =
                    self.stack.get(self.stack.len().saturating_sub(2))
                {
                    let object = handle.borrow();

                    match &*object {
                        Object::Array(_) => {
                            drop(object);
                            self.op_get_array_index()?;
                        }

                        Object::Tuple(_) => {
                            drop(object);
                            self.op_get_tuple_index()?;
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

            OpCode::SetIndex => {
                if let Some(Value::Object(handle)) =
                    self.stack.get(self.stack.len().saturating_sub(3))
                {
                    let object = handle.borrow();

                    match &*object {
                        Object::Array(_) => {
                            drop(object);
                            self.op_set_array_index()?;
                        }

                        Object::Tuple(_) => {
                            return Err(RuntimeError::ImmutableValue("tuple"));
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

            OpCode::ArrayLength => {
                self.op_array_length()?;
                Ok(false)
            }

            OpCode::ArrayPush => {
                self.op_array_push()?;
                Ok(false)
            }

            OpCode::ArrayPop => {
                self.op_array_pop()?;
                Ok(false)
            }

            OpCode::ArrayInsert => {
                self.op_array_insert()?;
                Ok(false)
            }

            OpCode::ArrayRemove => {
                self.op_array_remove()?;
                Ok(false)
            }

            OpCode::ArrayClear => {
                self.op_array_clear()?;
                Ok(false)
            }

            OpCode::ArrayContains => {
                self.op_array_contains()?;
                Ok(false)
            }

            // ========================================================
            // OBJECTS
            // ========================================================
            OpCode::Object => {
                let pair_count = self.read_byte()? as usize;

                self.op_object(pair_count)?;
                Ok(false)
            }

            // ========================================================
            // ITERATORS
            // ========================================================
            OpCode::GetIterator => {
                self.op_get_iterator()?;
                Ok(false)
            }

            OpCode::IteratorHasNext => {
                self.op_iterator_has_next()?;
                Ok(false)
            }

            OpCode::IteratorNext => {
                self.op_iterator_next()?;
                Ok(false)
            }

            // ========================================================
            // PROPERTIES
            // ========================================================
            OpCode::GetProperty => {
                self.get_property()?;
                Ok(false)
            }

            OpCode::SetProperty => {
                self.set_property()?;
                Ok(false)
            }

            // ========================================================
            // METHOD CALLS
            // ========================================================
            OpCode::InvokeMethod => {
                let method_constant = self.read_byte()? as usize;
                let arg_count = self.read_byte()? as usize;

                self.op_invoke_method(method_constant, arg_count)?;
                Ok(false)
            }

            OpCode::InvokeBaseMethod => {
                let method_constant = self.read_byte()? as usize;
                let arg_count = self.read_byte()? as usize;

                self.op_invoke_base_method(method_constant, arg_count)?;
                Ok(false)
            }

            // ========================================================
            // CLASSES / INTERFACES
            // ========================================================
            OpCode::Class => {
                let base_count = self.read_byte()? as usize;
                let method_count = self.read_byte()? as usize;

                self.op_class(base_count, method_count)?;
                Ok(false)
            }

            OpCode::NewInstance => {
                let arg_count = self.read_byte()? as usize;

                self.op_new_instance(arg_count)?;
                Ok(false)
            }

            OpCode::Interface => {
                let base_count = self.read_byte()? as usize;
                let method_count = self.read_byte()? as usize;

                self.op_interface(base_count, method_count)?;
                Ok(false)
            }

            // ========================================================
            // EXCEPTIONS
            // ========================================================
            OpCode::PushExceptionHandler => {
                self.op_push_exception_handler()?;
                Ok(false)
            }

            OpCode::PopExceptionHandler => {
                self.op_pop_exception_handler()?;
                Ok(false)
            }

            OpCode::Throw => {
                self.op_throw()?;
                Ok(false)
            }

            OpCode::FinallyEnd => {
                self.op_finally_end()?;
                Ok(false)
            }

            // ========================================================
            // RETURN / HALT
            // ========================================================
            OpCode::Return => {
                self.execute_return()?;
                Ok(self.frames.is_empty())
            }

            OpCode::Halt => Ok(true),

            // ========================================================
            // SUPER-INSTRUCTIONS
            // ========================================================
            OpCode::SetLocalPop => {
                self.set_local_pop()?;
                Ok(false)
            }

            OpCode::AddLocalConst => {
                self.add_local_const()?;
                Ok(false)
            }

            OpCode::LessLocalConst => {
                self.less_local_const()?;
                Ok(false)
            }

            OpCode::JumpIfFalsePop => {
                self.jump_if_false_pop()?;
                Ok(false)
            }

            OpCode::LessLocalConstJump => {
                self.less_local_const_jump()?;
                Ok(false)
            }

            OpCode::LoopLessAddLocalConst => {
                self.loop_less_add_local_const()?;
                Ok(false)
            }

            OpCode::AddLocalLocal => {
                self.add_local_local()?;
                Ok(false)
            }
        }
    }
}

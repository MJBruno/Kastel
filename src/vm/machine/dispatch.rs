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
        #[cfg(feature = "profile")]
        self.profile_instruction(instruction);

        let opcode =
            OpCode::from_byte(instruction).map_err(|_| RuntimeError::InvalidOpcode(instruction))?;

        match opcode {
            // ========================================================
            // CONSTANTS / GLOBALS
            // ========================================================
            OpCode::Constant => {
                let constant = self.read_constant_byte()?;
                self.push(constant);
            }

            OpCode::DefineGlobal => {
                self.define_global()?;
            }

            OpCode::GetGlobal => {
                self.get_global()?;
            }

            OpCode::SetGlobal => {
                self.set_global()?;
            }

            OpCode::GetLocal => {
                self.get_local()?;
            }

            OpCode::SetLocal => {
                self.set_local()?;
            }

            OpCode::SetLocalPop => {
                self.set_local_pop()?;
            }

            OpCode::AddLocalConst => {
                self.add_local_const()?;
            }
            OpCode::LessLocalConstJump => {
                self.less_local_const_jump()?;
            }

            OpCode::LoopLessAddLocalConst => {
                self.loop_less_add_local_const()?;
            }

            OpCode::GetUpvalue => {
                let index = self.read_byte()? as usize;
                self.get_upvalue(index)?;
            }

            OpCode::SetUpvalue => {
                let index = self.read_byte()? as usize;
                self.set_upvalue(index)?;
            }

            // ========================================================
            // LITERALS
            // ========================================================
            OpCode::True => {
                self.push(Value::Boolean(true));
            }

            OpCode::False => {
                self.push(Value::Boolean(false));
            }

            OpCode::Nil => {
                self.push(Value::Nil);
            }

            // ========================================================
            // ARITHMETIC
            // ========================================================
            OpCode::Add => {
                self.add()?;
            }

            OpCode::Subtract => {
                self.numeric_binary(NumericOp::Subtract)?;
            }

            OpCode::Multiply => {
                self.numeric_binary(NumericOp::Multiply)?;
            }

            OpCode::Divide => {
                self.numeric_binary(NumericOp::Divide)?;
            }

            OpCode::Modulo => {
                self.numeric_binary(NumericOp::Modulo)?;
            }

            OpCode::Negate => {
                self.negate()?;
            }

            // ========================================================
            // BITWISE
            // ========================================================
            OpCode::BitAnd => {
                self.bitwise_binary(0)?;
            }

            OpCode::BitOr => {
                self.bitwise_binary(1)?;
            }

            OpCode::BitXor => {
                self.bitwise_binary(2)?;
            }

            OpCode::BitNot => {
                self.bitwise_not()?;
            }

            OpCode::ShiftLeft => {
                self.shift(true)?;
            }

            OpCode::ShiftRight => {
                self.shift(false)?;
            }

            // ========================================================
            // COMPARISON
            // ========================================================
            OpCode::Equal => {
                let b = self.pop()?;
                let a = self.pop()?;

                self.push(Value::Boolean(Value::equals(a, b)));
            }

            OpCode::Greater => {
                self.compare(ComparisonOp::Greater)?;
            }

            OpCode::Less => {
                self.compare(ComparisonOp::Less)?;
            }

            OpCode::LessLocalConst => {
                self.less_local_const()?;
            }

            OpCode::Not => {
                self.not()?;
            }

            OpCode::Is => {
                let right = self.pop()?;
                let left = self.pop()?;

                let result = Self::is_value_instance_of(&left, &right)?;

                self.push(Value::Boolean(result));
            }

            // ========================================================
            // OBJECT / PROPERTY
            // ========================================================
            OpCode::GetProperty => {
                self.get_property()?;
            }

            OpCode::SetProperty => {
                self.set_property()?;
            }

            // ========================================================
            // ARRAY / OBJECT
            // ========================================================
            OpCode::Array => {
                let count = self.read_byte()? as usize;
                self.op_array(count)?;
            }

            OpCode::Object => {
                let pair_count = self.read_byte()? as usize;
                self.op_object(pair_count)?;
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

            OpCode::ArrayLength => {
                self.op_array_length()?;
            }

            OpCode::ArrayPush => {
                self.op_array_push()?;
            }

            OpCode::ArrayPop => {
                self.op_array_pop()?;
            }

            OpCode::ArrayInsert => {
                self.op_array_insert()?;
            }

            OpCode::ArrayRemove => {
                self.op_array_remove()?;
            }

            OpCode::ArrayClear => {
                self.op_array_clear()?;
            }

            OpCode::ArrayContains => {
                self.op_array_contains()?;
            }

            // ========================================================
            // ITERATORS
            // ========================================================
            OpCode::GetIterator => {
                self.op_get_iterator()?;
            }

            OpCode::IteratorHasNext => {
                self.op_iterator_has_next()?;
            }

            OpCode::IteratorNext => {
                self.op_iterator_next()?;
            }

            // ========================================================
            // FUNCTIONS / CLOSURES
            // ========================================================
            OpCode::Closure => {
                self.op_closure()?;
            }

            OpCode::Call => {
                let arg_count = self.read_byte()? as usize;

                self.execute_call(arg_count)?;
            }

            OpCode::InvokeMethod => {
                let method_constant = self.read_byte()? as usize;

                let arg_count = self.read_byte()? as usize;

                self.op_invoke_method(method_constant, arg_count)?;
            }

            OpCode::InvokeBaseMethod => {
                let method_constant = self.read_byte()? as usize;

                let arg_count = self.read_byte()? as usize;

                self.op_invoke_base_method(method_constant, arg_count)?;
            }

            // ========================================================
            // CLASS / INTERFACE
            // ========================================================
            OpCode::Interface => {
                let base_count = self.read_byte()? as usize;

                let method_count = self.read_byte()? as usize;

                self.op_interface(base_count, method_count)?;
            }

            OpCode::Class => {
                let base_count = self.read_byte()? as usize;

                let method_count = self.read_byte()? as usize;

                self.op_class(base_count, method_count)?;
            }

            OpCode::NewInstance => {
                let arg_count = self.read_byte()? as usize;

                self.op_new_instance(arg_count)?;
            }

            // ========================================================
            // MODULES
            // ========================================================
            OpCode::Import => {
                self.import_module()?;
            }

            // ========================================================
            // CONTROL FLOW
            // ========================================================
            OpCode::Jump => {
                self.jump()?;
            }

            OpCode::JumpIfFalse => {
                self.jump_if_false()?;
            }

            OpCode::JumpIfFalsePop => {
                self.jump_if_false_pop()?;
            }

            OpCode::Loop => {
                self.loop_back()?;
            }

            OpCode::Pop => {
                self.pop()?;
            }

            // ========================================================
            // RETURN / HALT
            // ========================================================
            OpCode::Return => {
                self.execute_return()?;

                return Ok(self.frames.is_empty());
            }

            OpCode::Halt => {
                return Ok(true);
            }

            // ========================================================
            // EXCEPTIONS
            // ========================================================
            OpCode::PushExceptionHandler => {
                self.op_push_exception_handler()?;
            }

            OpCode::PopExceptionHandler => {
                self.op_pop_exception_handler()?;
            }

            OpCode::Throw => {
                self.op_throw()?;
            }

            OpCode::FinallyEnd => {
                self.op_finally_end()?;
            }
        }

        Ok(false)
    }
}

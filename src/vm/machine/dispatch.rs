use super::VirtualMachine;

use crate::{
    bytecode::chunk::OpCode,
    error::runtime_error::RuntimeError,
    runtime::value::{ComparisonOp, NumericOp, Value},
};

impl VirtualMachine {
    pub(crate) fn dispatch(
        &mut self,
        instruction: u8,
    ) -> Result<bool, RuntimeError> {
        let opcode =
            OpCode::try_from(instruction)
                .map_err(|_| {
                    RuntimeError::InvalidOpcode(
                        instruction
                    )
                })?;

        match opcode {
            OpCode::Constant => {
                let constant =
                    self.read_constant_byte()?;

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

            OpCode::GetUpvalue => {
                let index =
                    self.read_byte()? as usize;

                self.get_upvalue(index)?;
            }

            OpCode::SetUpvalue => {
                let index =
                    self.read_byte()? as usize;

                self.set_upvalue(index)?;
            }

            OpCode::True => {
                self.push(Value::Boolean(true));
            }

            OpCode::False => {
                self.push(Value::Boolean(false));
            }

            OpCode::Nil => {
                self.push(Value::Nil);
            }

            OpCode::Add => {
                self.add()?;
            }

            OpCode::Subtract => {
                self.numeric_binary(
                    NumericOp::Subtract
                )?;
            }

            OpCode::Multiply => {
                self.numeric_binary(
                    NumericOp::Multiply
                )?;
            }

            OpCode::Divide => {
                self.numeric_binary(
                    NumericOp::Divide
                )?;
            }

            OpCode::Modulo => {
                self.numeric_binary(
                    NumericOp::Modulo
                )?;
            }

            OpCode::Negate => {
                self.negate()?;
            }

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

            OpCode::Equal => {
                let b = self.pop()?;
                let a = self.pop()?;

                self.push(
                    Value::Boolean(
                        Value::equals(a, b)
                    )
                );
            }

            OpCode::Greater => {
                self.compare(
                    ComparisonOp::Greater
                )?;
            }

            OpCode::Less => {
                self.compare(
                    ComparisonOp::Less
                )?;
            }

            OpCode::Not => {
                self.not()?;
            }

            OpCode::GetProperty => {
                self.get_property()?;
            }

            OpCode::SetProperty => {
                self.set_property()?;
            }

            OpCode::Array => {
                let count =
                    self.read_byte()? as usize;

                self.op_array(count)?;
            }

            OpCode::Object => {
                let pair_count =
                    self.read_byte()? as usize;

                self.op_object(pair_count)?;
            }

            OpCode::GetIndex => {
                self.op_get_index()?;
            }

            OpCode::SetIndex => {
                self.op_set_index()?;
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

            OpCode::GetIterator => {
                self.op_get_iterator()?;
            }

            OpCode::IteratorHasNext => {
                self.op_iterator_has_next()?;
            }

            OpCode::IteratorNext => {
                self.op_iterator_next()?;
            }

            OpCode::Closure => {
                self.op_closure()?;
            }

            OpCode::Call => {
                let arg_count =
                    self.read_byte()? as usize;

                self.execute_call(arg_count)?;
            }

            OpCode::InvokeMethod => {
                let method_constant =
                    self.read_byte()? as usize;

                let arg_count =
                    self.read_byte()? as usize;

                self.op_invoke_method(
                    method_constant,
                    arg_count,
                )?;
            }

            OpCode::Import => {
                self.import_module()?;
            }

            OpCode::Jump => {
                self.jump()?;
            }

            OpCode::JumpIfFalse => {
                self.jump_if_false()?;
            }

            OpCode::Loop => {
                self.loop_back()?;
            }

            OpCode::Pop => {
                self.pop()?;
            }

            OpCode::Return => {
                self.execute_return()?;

                return Ok(self.frames.is_empty());
            }

            OpCode::Halt => {
                return Ok(true);
            }
        }

        Ok(false)
    }
}
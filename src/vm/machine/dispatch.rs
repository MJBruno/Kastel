use super::VirtualMachine;
use crate::bytecode::chunk::OpCode;
use crate::error::runtime_error::RuntimeError;
use crate::runtime::value::{ComparisonOp, NumericOp, Value};

impl VirtualMachine {
    pub(crate) fn dispatch(&mut self, instruction: u8) -> Result<bool, RuntimeError> {
        match instruction {
            x if x == OpCode::Constant.into() => {
                let constant = self.read_constant_byte()?;
                self.push(constant);
            }
            x if x == OpCode::DefineGlobal.into() => {
                self.define_global()?;
            }
            x if x == OpCode::GetGlobal.into() => {
                self.get_global()?;
            }
            x if x == OpCode::SetGlobal.into() => {
                self.set_global()?;
            }
            x if x == OpCode::GetLocal.into() => {
                self.get_local()?;
            }
            x if x == OpCode::SetLocal.into() => {
                self.set_local()?;
            }

            x if x == OpCode::True.into() => self.push(Value::Boolean(true)),
            x if x == OpCode::False.into() => self.push(Value::Boolean(false)),
            x if x == OpCode::Nil.into() => self.push(Value::Nil),

            x if x == OpCode::Add.into() => self.add()?,
            x if x == OpCode::Subtract.into() => self.numeric_binary(NumericOp::Subtract)?,
            x if x == OpCode::Multiply.into() => self.numeric_binary(NumericOp::Multiply)?,
            x if x == OpCode::Divide.into() => self.numeric_binary(NumericOp::Divide)?,
            x if x == OpCode::Modulo.into() => self.numeric_binary(NumericOp::Modulo)?,
            x if x == OpCode::Negate.into() => self.negate()?,

            x if x == OpCode::BitAnd.into() => self.bitwise_binary(0)?,
            x if x == OpCode::BitOr.into() => self.bitwise_binary(1)?,
            x if x == OpCode::BitXor.into() => self.bitwise_binary(2)?,
            x if x == OpCode::BitNot.into() => self.bitwise_not()?,
            x if x == OpCode::ShiftLeft.into() => self.shift(true)?,
            x if x == OpCode::ShiftRight.into() => self.shift(false)?,

            x if x == OpCode::Equal.into() => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(Value::Boolean(Value::equals(a, b)));
            }
            x if x == OpCode::Greater.into() => self.compare(ComparisonOp::Greater)?,
            x if x == OpCode::Less.into() => self.compare(ComparisonOp::Less)?,
            x if x == OpCode::Not.into() => self.not()?,

            x if x == OpCode::GetProperty.into() => self.get_property()?,
            x if x == OpCode::SetProperty.into() => self.set_property()?,

            x if x == OpCode::Array.into() => {
                let count = self.read_byte()? as usize;
                self.op_array(count)?;
            }
            x if x == OpCode::Object.into() => {
                let pairs = self.read_byte()? as usize;
                self.op_object(pairs)?;
            }
            x if x == OpCode::GetIterator.into() => self.op_get_iterator()?,
            x if x == OpCode::IteratorHasNext.into() => self.op_iterator_has_next()?,
            x if x == OpCode::IteratorNext.into() => self.op_iterator_next()?,
            x if x == OpCode::GetIndex.into() => self.op_get_index()?,
            x if x == OpCode::SetIndex.into() => self.op_set_index()?,
            x if x == OpCode::ArrayLength.into() => self.op_array_length()?,
            x if x == OpCode::ArrayPush.into() => self.op_array_push()?,
            x if x == OpCode::ArrayPop.into() => self.op_array_pop()?,
            x if x == OpCode::ArrayInsert.into() => self.op_array_insert()?,
            x if x == OpCode::ArrayRemove.into() => self.op_array_remove()?,
            x if x == OpCode::ArrayClear.into() => self.op_array_clear()?,
            x if x == OpCode::ArrayContains.into() => self.op_array_contains()?,

            x if x == OpCode::Import.into() => self.import_module()?,
            x if x == OpCode::Closure.into() => self.op_closure()?,
            x if x == OpCode::SetUpvalue.into() => {
                let index = self.read_byte()? as usize;
                self.set_upvalue(index)?;
            }
            x if x == OpCode::GetUpvalue.into() => {
                let index = self.read_byte()? as usize;
                self.get_upvalue(index)?;
            }

            x if x == OpCode::Jump.into() => self.jump()?,
            x if x == OpCode::JumpIfFalse.into() => self.jump_if_false()?,
            x if x == OpCode::Loop.into() => self.loop_back()?,

            x if x == OpCode::Call.into() => {
                let arg_count = self.read_byte()? as usize;
                self.execute_call(arg_count)?;
            }

            x if x == OpCode::Pop.into() => {
                self.pop()?;
            }
            x if x == OpCode::Return.into() => {
                self.execute_return()?;
                return Ok(self.frames.is_empty());
            }
            x if x == OpCode::Halt.into() => return Ok(true),
            _ => return Err(RuntimeError::InvalidOpcode(instruction)),
        }
        Ok(false)
    }
}

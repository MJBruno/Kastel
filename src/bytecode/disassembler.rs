use std::rc::Rc;

use crate::runtime::object::Object;
use crate::runtime::value::Value;

use super::chunk::Chunk;
use super::opcode::OpCode;

impl Chunk {
    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        if offset >= self.code.len() {
            println!("{offset:04} <EOF>");
            return offset;
        }

        print!("{offset:04} ");

        let instruction = self.code[offset];

        match instruction {
            // =====================================================
            // CONSTANTS
            // =====================================================
            x if x == OpCode::Constant.into() => self.constant_instruction("OP_CONSTANT", offset),

            x if x == OpCode::DefineGlobal.into() => {
                self.constant_instruction("OP_DEFINE_GLOBAL", offset)
            }

            x if x == OpCode::GetGlobal.into() => {
                self.constant_instruction("OP_GET_GLOBAL", offset)
            }

            // =====================================================
            // LOCALS / UPVALUES
            // =====================================================
            x if x == OpCode::GetLocal.into() => self.byte_instruction("OP_GET_LOCAL", offset),

            x if x == OpCode::SetLocal.into() => self.byte_instruction("OP_SET_LOCAL", offset),

            x if x == OpCode::GetUpvalue.into() => self.byte_instruction("OP_GET_UPVALUE", offset),

            x if x == OpCode::SetUpvalue.into() => self.byte_instruction("OP_SET_UPVALUE", offset),
            x if x == OpCode::Import.into() => self.constant_instruction("OP_IMPORT", offset),

            x if x == OpCode::GetProperty.into() => {
                self.constant_instruction("OP_GET_PROPERTY", offset)
            }

            x if x == OpCode::SetProperty.into() => {
                self.constant_instruction("OP_SET_PROPERTY", offset)
            }
            // =====================================================
            // ARITHMETIC
            // =====================================================
            x if x == OpCode::Add.into() => self.simple_instruction("OP_ADD", offset),

            x if x == OpCode::Subtract.into() => self.simple_instruction("OP_SUBTRACT", offset),

            x if x == OpCode::Multiply.into() => self.simple_instruction("OP_MULTIPLY", offset),

            x if x == OpCode::Divide.into() => self.simple_instruction("OP_DIVIDE", offset),

            x if x == OpCode::Modulo.into() => self.simple_instruction("OP_MODULO", offset),

            x if x == OpCode::Negate.into() => self.simple_instruction("OP_NEGATE", offset),

            x if x == OpCode::BitAnd.into() => self.simple_instruction("OP_BIT_AND", offset),
            x if x == OpCode::BitOr.into() => self.simple_instruction("OP_BIT_OR", offset),
            x if x == OpCode::BitXor.into() => self.simple_instruction("OP_BIT_XOR", offset),
            x if x == OpCode::BitNot.into() => self.simple_instruction("OP_BIT_NOT", offset),
            x if x == OpCode::ShiftLeft.into() => self.simple_instruction("OP_SHIFT_LEFT", offset),
            x if x == OpCode::ShiftRight.into() => {
                self.simple_instruction("OP_SHIFT_RIGHT", offset)
            }

            // =====================================================
            // COMPARISON
            // =====================================================
            x if x == OpCode::Equal.into() => self.simple_instruction("OP_EQUAL", offset),

            x if x == OpCode::Greater.into() => self.simple_instruction("OP_GREATER", offset),

            x if x == OpCode::Less.into() => self.simple_instruction("OP_LESS", offset),

            x if x == OpCode::Not.into() => self.simple_instruction("OP_NOT", offset),

            // =====================================================
            // LITERALS
            // =====================================================
            x if x == OpCode::Nil.into() => self.simple_instruction("OP_NIL", offset),

            x if x == OpCode::True.into() => self.simple_instruction("OP_TRUE", offset),

            x if x == OpCode::False.into() => self.simple_instruction("OP_FALSE", offset),

            // =====================================================
            // GLOBALS
            // =====================================================
            x if x == OpCode::SetGlobal.into() => {
                self.constant_instruction("OP_SET_GLOBAL", offset)
            }

            // =====================================================
            // CONTROL FLOW
            // =====================================================
            x if x == OpCode::JumpIfFalse.into() => self.jump_instruction("OP_JUMP", offset, false),

            x if x == OpCode::Jump.into() => self.jump_instruction("OP_JUMP", offset, false),

            x if x == OpCode::Loop.into() => self.jump_instruction("OP_LOOP", offset, true),

            // =====================================================
            // STACK / CALL
            // =====================================================
            x if x == OpCode::Pop.into() => self.simple_instruction("OP_POP", offset),

            x if x == OpCode::Call.into() => self.byte_instruction("OP_CALL", offset),

            // =====================================================
            // ARRAYS
            // =====================================================
            x if x == OpCode::Array.into() => self.byte_instruction("OP_ARRAY", offset),

            x if x == OpCode::Object.into() => self.byte_instruction("OP_OBJECT", offset),

            x if x == OpCode::GetIterator.into() => {
                self.simple_instruction("OP_GET_ITERATOR", offset)
            }

            x if x == OpCode::IteratorHasNext.into() => {
                self.simple_instruction("OP_ITERATOR_HAS_NEXT", offset)
            }

            x if x == OpCode::IteratorNext.into() => {
                self.simple_instruction("OP_ITERATOR_NEXT", offset)
            }

            x if x == OpCode::GetIndex.into() => self.simple_instruction("OP_GET_INDEX", offset),

            x if x == OpCode::SetIndex.into() => self.simple_instruction("OP_SET_INDEX", offset),

            x if x == OpCode::ArrayLength.into() => {
                self.simple_instruction("OP_ARRAY_LENGTH", offset)
            }

            x if x == OpCode::ArrayPush.into() => self.simple_instruction("OP_ARRAY_PUSH", offset),

            x if x == OpCode::ArrayPop.into() => self.simple_instruction("OP_ARRAY_POP", offset),

            x if x == OpCode::ArrayInsert.into() => {
                self.simple_instruction("OP_ARRAY_INSERT", offset)
            }

            x if x == OpCode::ArrayRemove.into() => {
                self.simple_instruction("OP_ARRAY_REMOVE", offset)
            }

            x if x == OpCode::ArrayClear.into() => {
                self.simple_instruction("OP_ARRAY_CLEAR", offset)
            }

            x if x == OpCode::ArrayContains.into() => {
                self.simple_instruction("OP_ARRAY_CONTAINS", offset)
            }

            // =====================================================
            // CLOSURES
            // =====================================================
            x if x == OpCode::Closure.into() => self.closure_instruction(offset),

            // =====================================================
            // RETURN / HALT
            // =====================================================
            x if x == OpCode::Return.into() => self.simple_instruction("OP_RETURN", offset),

            x if x == OpCode::Halt.into() => self.simple_instruction("OP_HALT", offset),

            _ => {
                println!("OP_UNKNOWN {instruction}");
                offset + 1
            }
        }
    }

    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{name}");
        offset + 1
    }

    fn byte_instruction(&self, name: &str, offset: usize) -> usize {
        if offset + 1 >= self.code.len() {
            println!("{name:<16} <missing operand>");
            return self.code.len();
        }

        let slot = self.code[offset + 1];

        println!("{name:<16} {:4}", slot);

        offset + 2
    }

    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        if offset + 1 >= self.code.len() {
            println!("{name:<16} <missing operand>");
            return self.code.len();
        }

        let constant_index = self.code[offset + 1] as usize;

        match self.constants.get(constant_index) {
            Some(constant) => {
                println!("{name:<16} {:4} '{constant}'", constant_index);
            }

            None => {
                println!("{name:<16} {:4} <invalid constant>", constant_index);
            }
        }

        offset + 2
    }

    fn jump_instruction(&self, name: &str, offset: usize, backward: bool) -> usize {
        if offset + 2 >= self.code.len() {
            println!("{:<20} <missing operand>", name);
            return self.code.len();
        }

        let high = self.code[offset + 1] as u16;
        let low = self.code[offset + 2] as u16;

        let jump = ((high << 8) | low) as usize;

        let target = if backward {
            offset + 3 - jump
        } else {
            offset + 3 + jump
        };

        println!("{:<20} {:4} -> {:04}", name, jump, target);

        offset + 3
    }

    fn closure_instruction(&self, offset: usize) -> usize {
        if offset + 1 >= self.code.len() {
            println!("{:<20} <missing function constant>", "OP_CLOSURE");
            return self.code.len();
        }

        let constant_index = self.code[offset + 1] as usize;

        match self.constants.get(constant_index) {
            Some(constant) => {
                println!("{:<20} {:4} '{constant}'", "OP_CLOSURE", constant_index);
            }

            None => {
                println!(
                    "{:<20} {:4} <invalid constant>",
                    "OP_CLOSURE", constant_index
                );
            }
        }

        let function = match self.constants.get(constant_index) {
            Some(Value::Object(handle)) => match &*handle.borrow() {
                Object::Function(function) => Rc::clone(function),
                _ => {
                    return offset + 2;
                }
            },
            _ => {
                return offset + 2;
            }
        };

        let mut next = offset + 2;

        for index in 0..function.upvalue_count {
            if next + 1 >= self.code.len() {
                println!("             upvalue[{index}] <missing operands>");
                return self.code.len();
            }

            let is_local = self.code[next];
            let upvalue_index = self.code[next + 1];

            println!(
                "             upvalue[{index}] {} {}",
                if is_local != 0 { "local" } else { "upvalue" },
                upvalue_index
            );

            next += 2;
        }

        next
    }
}
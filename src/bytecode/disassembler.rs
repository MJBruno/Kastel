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

        let opcode = match OpCode::from_byte(instruction) {
            Ok(opcode) => opcode,
            Err(()) => {
                println!("OP_UNKNOWN {instruction}");
                return offset + 1;
            }
        };

        if let OpCode::Constant = opcode {
            self.constant_instruction("OP_CONSTANT", offset)
        } else if let OpCode::None = opcode {
            self.simple_instruction("OP_NONE", offset)
        } else if let OpCode::True = opcode {
            self.simple_instruction("OP_TRUE", offset)
        } else if let OpCode::False = opcode {
            self.simple_instruction("OP_FALSE", offset)
        } else if let OpCode::Equal = opcode {
            self.simple_instruction("OP_EQUAL", offset)
        } else if let OpCode::Greater = opcode {
            self.simple_instruction("OP_GREATER", offset)
        } else if let OpCode::Less = opcode {
            self.simple_instruction("OP_LESS", offset)
        } else if let OpCode::Not = opcode {
            self.simple_instruction("OP_NOT", offset)
        } else if let OpCode::Is = opcode {
            self.constant_instruction("OP_IS", offset)
        } else if let OpCode::Add = opcode {
            self.simple_instruction("OP_ADD", offset)
        } else if let OpCode::Subtract = opcode {
            self.simple_instruction("OP_SUBTRACT", offset)
        } else if let OpCode::Multiply = opcode {
            self.simple_instruction("OP_MULTIPLY", offset)
        } else if let OpCode::Divide = opcode {
            self.simple_instruction("OP_DIVIDE", offset)
        } else if let OpCode::Modulo = opcode {
            self.simple_instruction("OP_MODULO", offset)
        } else if let OpCode::Negate = opcode {
            self.simple_instruction("OP_NEGATE", offset)
        } else if let OpCode::BitAnd = opcode {
            self.simple_instruction("OP_BIT_AND", offset)
        } else if let OpCode::BitOr = opcode {
            self.simple_instruction("OP_BIT_OR", offset)
        } else if let OpCode::BitXor = opcode {
            self.simple_instruction("OP_BIT_XOR", offset)
        } else if let OpCode::BitNot = opcode {
            self.simple_instruction("OP_BIT_NOT", offset)
        } else if let OpCode::ShiftLeft = opcode {
            self.simple_instruction("OP_SHIFT_LEFT", offset)
        } else if let OpCode::ShiftRight = opcode {
            self.simple_instruction("OP_SHIFT_RIGHT", offset)
        } else if let OpCode::DefineGlobal = opcode {
            self.constant_instruction("OP_DEFINE_GLOBAL", offset)
        } else if let OpCode::SetGlobal = opcode {
            self.constant_instruction("OP_SET_GLOBAL", offset)
        } else if let OpCode::GetGlobal = opcode {
            self.constant_instruction("OP_GET_GLOBAL", offset)
        } else if let OpCode::SetLocal = opcode {
            self.byte_instruction("OP_SET_LOCAL", offset)
        } else if let OpCode::GetLocal = opcode {
            self.byte_instruction("OP_GET_LOCAL", offset)
        } else if let OpCode::GetUpvalue = opcode {
            self.byte_instruction("OP_GET_UPVALUE", offset)
        } else if let OpCode::SetUpvalue = opcode {
            self.byte_instruction("OP_SET_UPVALUE", offset)
        } else if let OpCode::Import = opcode {
            self.constant_instruction("OP_IMPORT", offset)
        } else if let OpCode::ImportAll = opcode {
            self.constant_instruction("OP_IMPORT_ALL", offset)
        } else if let OpCode::JumpIfFalse = opcode {
            self.jump_instruction("OP_JUMP_IF_FALSE", offset, false)
        } else if let OpCode::Jump = opcode {
            self.jump_instruction("OP_JUMP", offset, false)
        } else if let OpCode::Loop = opcode {
            self.jump_instruction("OP_LOOP", offset, true)
        } else if let OpCode::Pop = opcode {
            self.simple_instruction("OP_POP", offset)
        } else if let OpCode::Call = opcode {
            self.byte_instruction("OP_CALL", offset)
        } else if let OpCode::Array = opcode {
            self.byte_instruction("OP_ARRAY", offset)
        } else if let OpCode::Tuple = opcode {
            self.byte_instruction("OP_TUPLE", offset)
        } else if let OpCode::Wide = opcode {
            self.wide_instruction(offset)
        } else if let OpCode::Record = opcode {
            self.byte_instruction("OP_RECORD", offset)
        } else if let OpCode::Overload = opcode {
            self.constant_instruction("OP_OVERLOAD", offset)
        } else if let OpCode::OverloadLocal = opcode {
            self.byte_instruction("OP_OVERLOAD_LOCAL", offset)
        } else if let OpCode::Object = opcode {
            self.byte_instruction("OP_OBJECT", offset)
        } else if let OpCode::GetIndex = opcode {
            self.simple_instruction("OP_GET_INDEX", offset)
        } else if let OpCode::SetIndex = opcode {
            self.simple_instruction("OP_SET_INDEX", offset)
        } else if let OpCode::ArrayLength = opcode {
            self.simple_instruction("OP_ARRAY_LENGTH", offset)
        } else if let OpCode::GetIterator = opcode {
            self.simple_instruction("OP_GET_ITERATOR", offset)
        } else if let OpCode::IteratorHasNext = opcode {
            self.simple_instruction("OP_ITERATOR_HAS_NEXT", offset)
        } else if let OpCode::IteratorNext = opcode {
            self.simple_instruction("OP_ITERATOR_NEXT", offset)
        } else if let OpCode::Closure = opcode {
            self.closure_instruction(offset)
        } else if let OpCode::GetProperty = opcode {
            self.constant_instruction("OP_GET_PROPERTY", offset)
        } else if let OpCode::SetProperty = opcode {
            self.constant_instruction("OP_SET_PROPERTY", offset)
        } else if let OpCode::InvokeMethod = opcode {
            self.constant_instruction("OP_INVOKE_METHOD", offset)
        } else if let OpCode::InvokeBaseMethod = opcode {
            self.constant_instruction("OP_REMOVED_INVOKE_BASE_METHOD", offset)
        } else if let OpCode::Class = opcode {
            self.five_byte_instruction("OP_CLASS", offset)
        } else if let OpCode::NewInstance = opcode {
            self.byte_instruction("OP_NEW_INSTANCE", offset)
        } else if let OpCode::Interface = opcode {
            self.constant_instruction("OP_INTERFACE", offset)
        } else if let OpCode::Enum = opcode {
            self.two_byte_instruction("OP_ENUM", offset)
        } else if let OpCode::PushExceptionHandler = opcode {
            self.exception_handler_instruction("OP_PUSH_EXCEPTION_HANDLER", offset)
        } else if let OpCode::PopExceptionHandler = opcode {
            self.simple_instruction("OP_POP_EXCEPTION_HANDLER", offset)
        } else if let OpCode::Throw = opcode {
            self.simple_instruction("OP_THROW", offset)
        } else if let OpCode::FinallyEnd = opcode {
            self.simple_instruction("OP_FINALLY_END", offset)
        } else if let OpCode::Try = opcode {
            self.simple_instruction("OP_TRY", offset)
        } else if let OpCode::Spawn = opcode {
            self.byte_instruction("OP_SPAWN", offset)
        } else if let OpCode::Yield = opcode {
            self.simple_instruction("OP_YIELD", offset)
        } else if let OpCode::Return = opcode {
            self.simple_instruction("OP_RETURN", offset)
        } else if let OpCode::Halt = opcode {
            self.simple_instruction("OP_HALT", offset)
        } else if let OpCode::AddLocalConst = opcode {
            self.byte_instruction("OP_ADD_LOCAL_CONST", offset)
        } else if let OpCode::LessLocalConst = opcode {
            self.byte_instruction("OP_LESS_LOCAL_CONST", offset)
        } else if let OpCode::JumpIfFalsePop = opcode {
            self.jump_instruction("OP_JUMP_IF_FALSE_POP", offset, false)
        } else if let OpCode::LessLocalConstJump = opcode {
            self.constant_jump_instruction("OP_LESS_LOCAL_CONST_JUMP", offset)
        } else if let OpCode::LoopLessAddLocalConst = opcode {
            self.loop_less_add_local_const_instruction(offset)
        } else {
            self.two_byte_instruction("OP_ADD_LOCAL_LOCAL", offset)
        }
    }

    // =============================================================
    // SIMPLE
    // =============================================================

    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{name}");
        offset + 1
    }

    // =============================================================
    // ONE BYTE OPERAND
    // =============================================================

    fn byte_instruction(&self, name: &str, offset: usize) -> usize {
        if offset + 1 >= self.code.len() {
            println!("{name:<24} <missing operand>");
            return self.code.len();
        }

        let operand = self.code[offset + 1];

        println!("{name:<24} {:4}", operand);

        offset + 2
    }

    /// `OP_CLASS` : bases, méthodes d'instance, méthodes statiques, champs
    /// statiques, membres privés (5 opérandes en un octet chacun).
    fn five_byte_instruction(&self, name: &str, offset: usize) -> usize {
        if offset + 5 >= self.code.len() {
            println!("{name:<24} <missing operands>");
            return self.code.len();
        }

        println!(
            "{name:<24} {:4} {:4} {:4} {:4} {:4}",
            self.code[offset + 1],
            self.code[offset + 2],
            self.code[offset + 3],
            self.code[offset + 4],
            self.code[offset + 5]
        );

        offset + 6
    }

    fn exception_handler_instruction(&self, name: &str, offset: usize) -> usize {
        if offset + 6 >= self.code.len() {
            println!("{name:<24} <missing operands>");
            return self.code.len();
        }

        let a = u16::from_be_bytes([self.code[offset + 1], self.code[offset + 2]]);
        let b = u16::from_be_bytes([self.code[offset + 3], self.code[offset + 4]]);
        let c = u16::from_be_bytes([self.code[offset + 5], self.code[offset + 6]]);
        println!("{name:<24} {a:04X} {b:04X} {c:04X}");
        offset + 7
    }

    fn two_byte_instruction(&self, name: &str, offset: usize) -> usize {
        if offset + 2 >= self.code.len() {
            println!("{name:<24} <missing operands>");
            return self.code.len();
        }

        let first = self.code[offset + 1];
        let second = self.code[offset + 2];

        println!("{name:<24} {:4} {:4}", first, second);

        offset + 3
    }

    // =============================================================
    // CONSTANT OPERAND
    // =============================================================

    /// `Wide <opcode> <haut> <bas> [autres opérandes]` : instruction à
    /// opérande constante sur 16 bits.
    fn wide_instruction(&self, offset: usize) -> usize {
        if offset + 3 >= self.code.len() {
            println!("{:<24} <missing operands>", "OP_WIDE");
            return self.code.len();
        }

        let inner = self.code[offset + 1];
        let index = ((self.code[offset + 2] as usize) << 8) | self.code[offset + 3] as usize;

        let inner_opcode = OpCode::from_byte(inner);

        let name = match inner_opcode {
            Ok(opcode) => format!("OP_WIDE {opcode:?}"),
            Err(()) => format!("OP_WIDE <opcode {inner}>"),
        };

        match self.constants.get(index) {
            Some(constant) => println!("{name:<24} {index:5} '{constant}'"),
            None => println!("{name:<24} {index:5} <invalid constant>"),
        }

        let mut next = offset + 4;

        match inner_opcode {
            // Nombre d'arguments, resté sur un octet.
            Ok(OpCode::InvokeMethod | OpCode::InvokeBaseMethod) => next += 1,

            // Paires (est_local, indice) des upvalues de la fonction.
            Ok(OpCode::Closure) => {
                if let Some(Value::Object(handle)) = self.constants.get(index)
                    && let Object::Function(function) = &*handle.borrow()
                {
                    next += 2 * function.upvalue_count;
                }
            }

            _ => {}
        }

        next
    }

    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        if offset + 1 >= self.code.len() {
            println!("{name:<24} <missing operand>");
            return self.code.len();
        }

        let constant_index = self.code[offset + 1] as usize;

        match self.constants.get(constant_index) {
            Some(constant) => {
                println!("{name:<24} {:4} '{constant}'", constant_index);
            }

            None => {
                println!("{name:<24} {:4} <invalid constant>", constant_index);
            }
        }

        offset + 2
    }

    // =============================================================
    // JUMP
    // =============================================================

    fn jump_instruction(&self, name: &str, offset: usize, backward: bool) -> usize {
        if offset + 2 >= self.code.len() {
            println!("{name:<24} <missing operands>");
            return self.code.len();
        }

        let high = self.code[offset + 1] as u16;
        let low = self.code[offset + 2] as u16;

        let jump = ((high << 8) | low) as usize;

        let target = if backward {
            match offset
                .checked_add(3)
                .and_then(|value| value.checked_sub(jump))
            {
                Some(target) => target,
                None => {
                    println!("{name:<24} {:4} -> <invalid target>", jump);
                    return offset + 3;
                }
            }
        } else {
            match offset
                .checked_add(3)
                .and_then(|value| value.checked_add(jump))
            {
                Some(target) => target,
                None => {
                    println!("{name:<24} {:4} -> <invalid target>", jump);
                    return offset + 3;
                }
            }
        };

        println!("{name:<24} {:4} -> {:04}", jump, target);

        offset + 3
    }

    // =============================================================
    // CONSTANT + JUMP
    // =============================================================

    fn constant_jump_instruction(&self, name: &str, offset: usize) -> usize {
        if offset + 3 >= self.code.len() {
            println!("{name:<24} <missing operands>");
            return self.code.len();
        }

        let constant_index = self.code[offset + 1] as usize;

        let high = self.code[offset + 2] as u16;
        let low = self.code[offset + 3] as u16;

        let jump = ((high << 8) | low) as usize;

        let target = match offset
            .checked_add(4)
            .and_then(|value| value.checked_add(jump))
        {
            Some(target) => target,
            None => {
                println!("{name:<24} {:4} <invalid jump target>", constant_index);
                return offset + 4;
            }
        };

        match self.constants.get(constant_index) {
            Some(constant) => {
                println!(
                    "{name:<24} const={:4} '{}' jump={:4} -> {:04}",
                    constant_index, constant, jump, target
                );
            }

            None => {
                println!(
                    "{name:<24} const={:4} <invalid constant> jump={:4} -> {:04}",
                    constant_index, jump, target
                );
            }
        }

        offset + 4
    }

    // =============================================================
    // SUPER-INSTRUCTION
    // =============================================================

    fn loop_less_add_local_const_instruction(&self, offset: usize) -> usize {
        if offset + 4 >= self.code.len() {
            println!("{:<24} <missing operands>", "OP_LOOP_LESS_ADD_LOCAL_CONST");
            return self.code.len();
        }

        let local = self.code[offset + 1];
        let constant_index = self.code[offset + 2] as usize;

        let high = self.code[offset + 3] as u16;
        let low = self.code[offset + 4] as u16;

        let jump = ((high << 8) | low) as usize;

        let target = match offset
            .checked_add(5)
            .and_then(|value| value.checked_sub(jump))
        {
            Some(target) => target,
            None => {
                println!(
                    "{:<24} local={} const={} jump={} -> <invalid target>",
                    "OP_LOOP_LESS_ADD_LOCAL_CONST", local, constant_index, jump
                );
                return offset + 5;
            }
        };

        match self.constants.get(constant_index) {
            Some(constant) => {
                println!(
                    "{:<24} local={} const={} '{}' jump={} -> {:04}",
                    "OP_LOOP_LESS_ADD_LOCAL_CONST", local, constant_index, constant, jump, target
                );
            }

            None => {
                println!(
                    "{:<24} local={} const={} <invalid constant> jump={} -> {:04}",
                    "OP_LOOP_LESS_ADD_LOCAL_CONST", local, constant_index, jump, target
                );
            }
        }

        offset + 5
    }

    // =============================================================
    // CLOSURE
    // =============================================================

    fn closure_instruction(&self, offset: usize) -> usize {
        if offset + 1 >= self.code.len() {
            println!("{:<24} <missing function constant>", "OP_CLOSURE");
            return self.code.len();
        }

        let constant_index = self.code[offset + 1] as usize;

        match self.constants.get(constant_index) {
            Some(constant) => {
                println!("{:<24} {:4} '{constant}'", "OP_CLOSURE", constant_index);
            }

            None => {
                println!(
                    "{:<24} {:4} <invalid constant>",
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

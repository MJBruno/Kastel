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

        match opcode {
            // =====================================================
            // CONSTANTS
            // =====================================================
            OpCode::Constant => self.constant_instruction("OP_CONSTANT", offset),

            // =====================================================
            // LITERALS
            // =====================================================
            OpCode::None => self.simple_instruction("OP_NONE", offset),

            OpCode::True => self.simple_instruction("OP_TRUE", offset),

            OpCode::False => self.simple_instruction("OP_FALSE", offset),

            // =====================================================
            // COMPARISON / LOGICAL
            // =====================================================
            OpCode::Equal => self.simple_instruction("OP_EQUAL", offset),

            OpCode::Greater => self.simple_instruction("OP_GREATER", offset),

            OpCode::Less => self.simple_instruction("OP_LESS", offset),

            OpCode::Not => self.simple_instruction("OP_NOT", offset),

            OpCode::Is => self.constant_instruction("OP_IS", offset),

            // =====================================================
            // ARITHMETIC
            // =====================================================
            OpCode::Add => self.simple_instruction("OP_ADD", offset),

            OpCode::Subtract => self.simple_instruction("OP_SUBTRACT", offset),

            OpCode::Multiply => self.simple_instruction("OP_MULTIPLY", offset),

            OpCode::Divide => self.simple_instruction("OP_DIVIDE", offset),

            OpCode::Modulo => self.simple_instruction("OP_MODULO", offset),

            OpCode::Negate => self.simple_instruction("OP_NEGATE", offset),

            // =====================================================
            // BITWISE
            // =====================================================
            OpCode::BitAnd => self.simple_instruction("OP_BIT_AND", offset),

            OpCode::BitOr => self.simple_instruction("OP_BIT_OR", offset),

            OpCode::BitXor => self.simple_instruction("OP_BIT_XOR", offset),

            OpCode::BitNot => self.simple_instruction("OP_BIT_NOT", offset),

            OpCode::ShiftLeft => self.simple_instruction("OP_SHIFT_LEFT", offset),

            OpCode::ShiftRight => self.simple_instruction("OP_SHIFT_RIGHT", offset),

            // =====================================================
            // GLOBALS
            // =====================================================
            OpCode::DefineGlobal => self.constant_instruction("OP_DEFINE_GLOBAL", offset),

            OpCode::SetGlobal => self.constant_instruction("OP_SET_GLOBAL", offset),

            OpCode::GetGlobal => self.constant_instruction("OP_GET_GLOBAL", offset),

            // =====================================================
            // LOCALS / UPVALUES
            // =====================================================
            OpCode::SetLocal => self.byte_instruction("OP_SET_LOCAL", offset),

            OpCode::GetLocal => self.byte_instruction("OP_GET_LOCAL", offset),

            OpCode::GetUpvalue => self.byte_instruction("OP_GET_UPVALUE", offset),

            OpCode::SetUpvalue => self.byte_instruction("OP_SET_UPVALUE", offset),

            // =====================================================
            // MODULES
            // =====================================================
            OpCode::Import => self.constant_instruction("OP_IMPORT", offset),

            OpCode::ImportAll => self.constant_instruction("OP_IMPORT_ALL", offset),

            // =====================================================
            // CONTROL FLOW
            // =====================================================
            OpCode::JumpIfFalse => self.jump_instruction("OP_JUMP_IF_FALSE", offset, false),

            OpCode::Jump => self.jump_instruction("OP_JUMP", offset, false),

            OpCode::Loop => self.jump_instruction("OP_LOOP", offset, true),

            // =====================================================
            // STACK / CALL
            // =====================================================
            OpCode::Pop => self.simple_instruction("OP_POP", offset),

            OpCode::Call => self.byte_instruction("OP_CALL", offset),

            // =====================================================
            // ARRAYS / OBJECTS
            // =====================================================
            OpCode::Array => self.byte_instruction("OP_ARRAY", offset),

            OpCode::Tuple => self.byte_instruction("OP_TUPLE", offset),

            OpCode::Wide => self.wide_instruction(offset),

            OpCode::Record => self.byte_instruction("OP_RECORD", offset),

            OpCode::Overload => self.constant_instruction("OP_OVERLOAD", offset),

            OpCode::OverloadLocal => self.byte_instruction("OP_OVERLOAD_LOCAL", offset),

            OpCode::Object => self.byte_instruction("OP_OBJECT", offset),

            OpCode::GetIndex => self.simple_instruction("OP_GET_INDEX", offset),

            OpCode::SetIndex => self.simple_instruction("OP_SET_INDEX", offset),

            OpCode::ArrayLength => self.simple_instruction("OP_ARRAY_LENGTH", offset),

            // =====================================================
            // ITERATORS
            // =====================================================
            OpCode::GetIterator => self.simple_instruction("OP_GET_ITERATOR", offset),

            OpCode::IteratorHasNext => self.simple_instruction("OP_ITERATOR_HAS_NEXT", offset),

            OpCode::IteratorNext => self.simple_instruction("OP_ITERATOR_NEXT", offset),

            // =====================================================
            // CLOSURES
            // =====================================================
            OpCode::Closure => self.closure_instruction(offset),

            // =====================================================
            // PROPERTIES
            // =====================================================
            OpCode::GetProperty => self.constant_instruction("OP_GET_PROPERTY", offset),

            OpCode::SetProperty => self.constant_instruction("OP_SET_PROPERTY", offset),

            // =====================================================
            // METHODS
            // =====================================================
            OpCode::InvokeMethod => self.constant_instruction("OP_INVOKE_METHOD", offset),

            OpCode::InvokeBaseMethod => self.constant_instruction("OP_INVOKE_BASE_METHOD", offset),

            // =====================================================
            // CLASSES / INSTANCES / INTERFACES
            // =====================================================
            OpCode::Class => self.three_byte_instruction("OP_CLASS", offset),

            OpCode::NewInstance => self.byte_instruction("OP_NEW_INSTANCE", offset),

            OpCode::Interface => self.constant_instruction("OP_INTERFACE", offset),

            OpCode::Enum => self.two_byte_instruction("OP_ENUM", offset),

            // =====================================================
            // EXCEPTIONS
            // =====================================================
            OpCode::PushExceptionHandler => {
                self.simple_instruction("OP_PUSH_EXCEPTION_HANDLER", offset)
            }

            OpCode::PopExceptionHandler => {
                self.simple_instruction("OP_POP_EXCEPTION_HANDLER", offset)
            }

            OpCode::Throw => self.simple_instruction("OP_THROW", offset),

            OpCode::FinallyEnd => self.simple_instruction("OP_FINALLY_END", offset),

            // =====================================================
            // RETURN / HALT
            // =====================================================
            OpCode::Return => self.simple_instruction("OP_RETURN", offset),

            OpCode::Halt => self.simple_instruction("OP_HALT", offset),

            // =====================================================
            // SUPER-INSTRUCTIONS
            // =====================================================
            OpCode::AddLocalConst => self.byte_instruction("OP_ADD_LOCAL_CONST", offset),

            OpCode::LessLocalConst => self.byte_instruction("OP_LESS_LOCAL_CONST", offset),

            OpCode::JumpIfFalsePop => self.jump_instruction("OP_JUMP_IF_FALSE_POP", offset, false),

            OpCode::LessLocalConstJump => {
                self.constant_jump_instruction("OP_LESS_LOCAL_CONST_JUMP", offset)
            }

            OpCode::LoopLessAddLocalConst => self.loop_less_add_local_const_instruction(offset),

            OpCode::AddLocalLocal => self.two_byte_instruction("OP_ADD_LOCAL_LOCAL", offset),
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

    // =============================================================
    // TWO BYTE OPERANDS
    // =============================================================

    /// `OP_CLASS bases méthodes membres_privés`
    fn three_byte_instruction(&self, name: &str, offset: usize) -> usize {
        if offset + 3 >= self.code.len() {
            println!("{name:<24} <missing operands>");
            return self.code.len();
        }

        println!(
            "{name:<24} {:4} {:4} {:4}",
            self.code[offset + 1],
            self.code[offset + 2],
            self.code[offset + 3]
        );

        offset + 4
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

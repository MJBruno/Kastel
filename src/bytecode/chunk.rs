use crate::runtime::value::Value;

 

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    Constant,
    Nil,
    True,
    False,

    // COMPARAISON
    Equal,
    Greater,
    Less,
    Not,

    // ARITHMETIQUE
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Negate,

    // SCOPE
    DefineGlobal,
    SetGlobal,
    GetGlobal,
    GetLocal,
    SetLocal,

    //
    Import,

    // CONTROLE
    JumpIfFalse,
    Jump,

    Pop,
    Loop,
    Call,

    //ARRAY
    Array,
    GetIndex,
    SetIndex,
    ArrayLength,
    ArrayPush,
    ArrayPop,
    ArrayInsert,
    ArrayRemove,
    ArrayClear,
    ArrayContains,

    // CLOSURES
    Closure,
    GetUpvalue,
    SetUpvalue,

    Return,
    Halt,
    GetProperty,
    SetProperty,
}

impl From<OpCode> for u8 {
    fn from(op: OpCode) -> Self {
        op as u8
    }
}

// Ceci stocke notre bytecode
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
        }
    }

    // poussé les donnée du bytecode
    pub fn write(&mut self, byte: u8) {
        self.code.push(byte);
    }

    /// <h3>Ajoute une constant dans le pool de constant</h3>
    /// <br>retourne l'index du constant actue pour faciliter OP_CONSTANT 0 <- index du constant
    /// <br>On peut utiliser: <code>let index = chunk.add_constant(42.0)</code>
    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    // pub fn disassemble(&self, name: &str) {
    //     println!("== {name} ==");

    //     let mut offset = 0;

    //     while offset < self.code.len() {
    //         offset = self.disassemble_instruction(offset);
    //     }
    // }

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
            Some(Value::Function(function)) => function,
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
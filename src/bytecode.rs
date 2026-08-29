use crate::value::Value;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    Constant,
    Nil,
    True,
    False,
    Equal,
    Greater,
    Less,
    Not,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Negate,
    DefineGlobal,
    SetGlobal,
    GetGlobal,
    GetLocal,
    SetLocal,
    JumpIfFalse,
    Jump,
    Closure,
    GetUpvalue,
    SetUpvalue,
    Pop,
    Print,
    Loop,
    Call,
    Array,
    GetIndex,
    SetIndex,
    ArrayLength,
    ArrayPush,
    ArrayPop,
    Return,
    Halt,
}

impl From<OpCode> for u8 {
    fn from(op: OpCode) -> Self {
        op as u8
    }
}

// Ceci stocke notre bytecode
#[derive(Debug, Clone)]
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

    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{offset:04} ");
        let instruction = self.code[offset];
        match instruction {
            x if x == OpCode::Constant.into() => self.constant_instruction("OP_CONSTANT", offset),
            x if x == OpCode::DefineGlobal.into() => {
                self.constant_instruction("OP_DEFINE_GLOBAL", offset)
            }
            x if x == OpCode::GetGlobal.into() => {
                self.constant_instruction("OP_GET_GLOBAL", offset)
            }
            x if x == OpCode::SetGlobal.into() => self.byte_instruction("OP_SET_GLOBAL", offset),
            x if x == OpCode::GetLocal.into() => self.byte_instruction("OP_GET_LOCAL", offset),
            x if x == OpCode::SetLocal.into() => self.constant_instruction("OP_SET_LOCAL", offset),
            x if x == OpCode::Add.into() => self.simple_instruction("OP_ADD", offset),
            x if x == OpCode::Subtract.into() => self.simple_instruction("OP_SUBTRACT", offset),
            x if x == OpCode::Multiply.into() => self.simple_instruction("OP_MULTIPLY", offset),
            x if x == OpCode::Divide.into() => self.simple_instruction("OP_DIVIDE", offset),
            x if x == OpCode::Modulo.into() => self.simple_instruction("OP_MODULO", offset),
            x if x == OpCode::True.into() => self.simple_instruction("OP_TRUE", offset),
            x if x == OpCode::False.into() => self.simple_instruction("OP_FALSE", offset),
            x if x == OpCode::Not.into() => self.simple_instruction("OP_NOT", offset),
            x if x == OpCode::Less.into() => self.simple_instruction("OP_LESS", offset),
            x if x == OpCode::Greater.into() => self.simple_instruction("OP_GREATER", offset),
            x if x == OpCode::Equal.into() => self.simple_instruction("OP_EQUAL", offset),
            x if x == OpCode::Nil.into() => self.simple_instruction("OP_NIL", offset),

            x if x == OpCode::Negate.into() => self.simple_instruction("OP_NEGATE", offset),
            x if x == OpCode::Loop.into() => self.simple_instruction("OP_LOOP", offset),
            x if x == OpCode::JumpIfFalse.into() => {
                self.byte_instruction("OP_JUMP_IF_FALSE", offset)
            }

            x if x == OpCode::Jump.into() => self.byte_instruction("OP_JUMP", offset),
            x if x == OpCode::Closure.into() => self.constant_instruction("OP_CLOSURE", offset),
            x if x == OpCode::GetUpvalue.into() => self.byte_instruction("OP_GET_UPVALUE", offset),
            x if x == OpCode::SetUpvalue.into() => self.byte_instruction("OP_SET_UPVALUE", offset),
            // Tableaux
            x if x == OpCode::Array.into() => self.byte_instruction("OP_ARRAY", offset),

            x if x == OpCode::GetIndex.into() => self.simple_instruction("OP_GET_INDEX", offset),

            x if x == OpCode::SetIndex.into() => self.simple_instruction("OP_SET_INDEX", offset),
            x if x == OpCode::Pop.into() => self.simple_instruction("OP_POP", offset),
            x if x == OpCode::Call.into() => self.byte_instruction("OP_CALL", offset),
            x if x == OpCode::Print.into() => self.simple_instruction("OP_PRINT", offset),
            x if x == OpCode::Return.into() => self.simple_instruction("OP_RETURN", offset),
            x if x == OpCode::Halt.into() => self.simple_instruction("OP_HALT", offset),
            _ => {
                panic!("Unknown opcode: {}", instruction);
            }
        }
    }

    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{name}");
        offset + 1
    }

    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        let constant_index = self.code[offset + 1] as usize;
        let constant = &self.constants[constant_index];
        println!("{name:<16} {:4} '{constant}'", constant_index);
        offset + 2
    }

    fn byte_instruction(&self, name: &str, offset: usize) -> usize {
        let slot = self.code[offset + 1];
        println!("{:<16} {:4}", name, slot);
        offset + 2
    }
}

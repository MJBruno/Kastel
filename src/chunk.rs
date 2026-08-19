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
    Negate,

    DefineGlobal,
    SetGlobal,
    GetGlobal,

    GetLocal,
    SetLocal,

    Return,
    Pop,
    Print,
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
    lines: Vec<usize>,
}
#[allow(dead_code)]
impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            lines: Vec::new(),
            constants: Vec::new(),
        }
    }

    // poussé les donnée du bytecode
    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    /// <h3>Ajoute une constant dans le pool de constant</h3>
    /// <br>retourne l'index du constant actue pour faciliter OP_CONSTANT 0 <- index du constant
    /// <br>On peut utiliser: <code>let index = chunk.add_constant(42.0)</code>
    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn write_opcode(&mut self, opcode: OpCode, line: usize) {
        self.write(opcode.into(), line);
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {name} ==");

        let mut offset = 0;

        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{offset:04} ");
        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("    | ");
        } else {
            print!("{:4} ", self.lines[offset]);
        }

        let instruction = self.code[offset];

        match instruction {
            x if x == OpCode::Constant.into() => self.constant_instruction("OP_CONSTANT", offset),
            x if x == OpCode::DefineGlobal.into() => {
                self.constant_instruction("OP_DEFINE_GLOBAL", offset)
            }
            x if x == OpCode::GetLocal.into() => self.byte_instruction("OP_GET_LOCAL", offset),
            x if x == OpCode::SetGlobal.into() => self.byte_instruction("OP_SET_GLOBAL", offset),
            x if x == OpCode::GetGlobal.into() => {
                self.constant_instruction("OP_GET_GLOBAL", offset)
            }
            x if x == OpCode::SetLocal.into() => self.constant_instruction("OP_SET_LOCAL", offset),

            x if x == OpCode::Return.into() => self.simple_instruction("OP_RETURN", offset),
            x if x == OpCode::Add.into() => self.simple_instruction("OP_ADD", offset),
            x if x == OpCode::Subtract.into() => self.simple_instruction("OP_SUBTRACT", offset),
            x if x == OpCode::Multiply.into() => self.simple_instruction("OP_MULTIPLY", offset),
            x if x == OpCode::Divide.into() => self.simple_instruction("OP_DIVIDE", offset),
            x if x == OpCode::True.into() => self.simple_instruction("OP_TRUE", offset),
            x if x == OpCode::False.into() => self.simple_instruction("OP_FALSE", offset),
            x if x == OpCode::Not.into() => self.simple_instruction("OP_NOT", offset),
            x if x == OpCode::Less.into() => self.simple_instruction("OP_LESS", offset),
            x if x == OpCode::Greater.into() => self.simple_instruction("OP_GREATER", offset),
            x if x == OpCode::Equal.into() => self.simple_instruction("OP_EQUAL", offset),
            x if x == OpCode::Nil.into() => self.simple_instruction("OP_NIL", offset),
            x if x == OpCode::Negate.into() => self.simple_instruction("OP_NEGATE", offset),
            x if x == OpCode::Pop.into() => self.simple_instruction("OP_POP", offset),
            x if x == OpCode::Print.into() => self.simple_instruction("OP_PRINT", offset),
            #[allow(unreachable_code)]
            _ => {
                panic!("Unknown opcode: {}", instruction);
                offset + 1
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

    fn trace_instruction(&self, offset: usize) {
        self.disassemble_instruction(offset);
    }
}

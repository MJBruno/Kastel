use std::collections::HashMap;

use crate::chunk::Chunk;
use crate::chunk::OpCode;

use crate::error_value::RuntimeError;
use crate::value::ComparisonOp;
use crate::value::NumericOp;
use crate::value::Value;
use crate::value::print_value;
#[allow(dead_code)]
pub struct VirtualMachine {
    pub chunk: Chunk,
    pub ip: usize,
    pub stack: Vec<Value>,
    pub globals: HashMap<String, Value>,
}
#[allow(dead_code)]
impl VirtualMachine {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            ip: 0,
            stack: Vec::new(),
            globals: HashMap::new(),
        }
    }

    //Lire les instructions(bytecode) dans le chunk
    fn read_byte(&mut self) -> u8 {
        let byte = self.chunk.code[self.ip];
        self.ip += 1;
        byte
    }

    //
    fn read_short(&mut self) -> u16 {
        let hight = self.read_byte() as u16;
        let low = self.read_byte() as u16;
        (hight << 8) | low
    }

    //Empiler la constante dans le pile
    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    //Dépiler la constante dans le pile
    fn pop(&mut self) -> Value {
        self.stack.pop().expect("Stack underflow")
    }

    fn peek(&self) -> &Value {
        self.stack.last().expect("Stack underflow")
    }

    ///Affiche l'état de notre pile
    fn print_stack(&self) {
        print!("          ");
        for value in &self.stack {
            print!(" [ {} ]", value);
        }
        println!()
    }

    // #[allow(unreachable_patterns)]
    pub fn run(&mut self) -> Result<(), RuntimeError> {
        loop {
            self.print_stack();
            self.chunk.disassemble_instruction(self.ip);

            let instruction = self.read_byte();

            match instruction {
                x if x == OpCode::Constant.into() => {
                    let index = self.read_byte() as usize;
                    let constant = self.chunk.constants[index].clone();
                    self.push(constant);
                }
                x if x == OpCode::Nil.into() => {
                    self.push(Value::Nil);
                }

                x if x == OpCode::DefineGlobal.into() => {
                    let index = self.read_byte() as usize;
                    let name = match &self.chunk.constants[index] {
                        Value::String(name) => name.clone(),
                        _ => return Err(RuntimeError::TypeError),
                    };
                    self.define_global(name);
                }
                x if x == OpCode::GetGlobal.into() => {
                    let index = self.read_byte() as usize;
                    let name = match &self.chunk.constants[index] {
                        Value::String(name) => name.clone(),
                        _ => return Err(RuntimeError::TypeError),
                    };
                    self.get_global(&name).expect("Value not exist");
                }

                x if x == OpCode::SetGlobal.into() => {
                    let index = self.read_byte() as usize;
                    let name = match &self.chunk.constants[index] {
                        Value::String(name) => name.clone(),
                        _ => return Err(RuntimeError::TypeError),
                    };
                    self.set_global(&name).expect("Value not exist");
                }

                x if x == OpCode::GetLocal.into() => {
                    let slot = self.read_byte() as usize;
                    self.get_local(slot);
                }

                x if x == OpCode::SetLocal.into() => {
                    let slot = self.read_byte() as usize;
                    self.set_local(slot);
                }

                x if x == OpCode::True.into() => {
                    self.push(Value::Boolean(true));
                }

                x if x == OpCode::False.into() => {
                    self.push(Value::Boolean(false));
                }

                x if x == OpCode::Add.into() => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = Value::binary_numeric_op(a, b, NumericOp::Add)?;
                    self.push(result);
                }
                x if x == OpCode::Subtract.into() => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = Value::binary_numeric_op(a, b, NumericOp::Subtract)?;
                    self.push(result);
                }
                x if x == OpCode::Multiply.into() => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = Value::binary_numeric_op(a, b, NumericOp::Multiply)?;
                    self.push(result);
                }
                x if x == OpCode::Divide.into() => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = Value::binary_numeric_op(a, b, NumericOp::Divide)?;
                    self.push(result);
                }
                x if x == OpCode::Modulo.into() => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = Value::binary_numeric_op(a, b, NumericOp::Modulo)?;
                    self.push(result);
                }
                x if x == OpCode::Equal.into() => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = Value::equals(a, b);
                    self.push(Value::Boolean(result));
                }
                x if x == OpCode::Greater.into() => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = Value::compare_numeric(a, b, ComparisonOp::Greater)?;
                    self.push(result);
                }
                x if x == OpCode::Less.into() => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = Value::compare_numeric(a, b, ComparisonOp::Less)?;
                    self.push(result);
                }
                x if x == OpCode::Negate.into() => {
                    let value = self.pop();
                    let result = Value::negate_values(value).expect("Opérand must be value");
                    self.push(result);
                }
                x if x == OpCode::Jump.into() => {
                    let offset = self.read_short();
                    self.ip += offset as usize;
                }
                x if x == OpCode::JumpIfFalse.into() => {
                    let offset = self.read_short();

                    if !self.peek().is_truthy() {
                        self.ip += offset as usize;
                    }
                }
                x if x == OpCode::Not.into() => {
                    let value = self.pop();
                    self.push(Value::Boolean(!value.is_truthy()));
                }
                x if x == OpCode::Pop.into() => {
                    let _ = self.pop();
                }
                x if x == OpCode::Print.into() => {
                    let value = self.pop();
                    print_value(value);
                }

                x if x == OpCode::Return.into() => {
                    return Ok(());
                }
                _ => panic!("Unknown opcode: {instruction}"),
            }
        }
    }

    //OP_DEFINE_GLOBAL
    fn define_global(&mut self, name: String) {
        let value = self.pop();
        self.globals.insert(name, value);
    }

    //OP_GET_GLOBAL
    fn get_global(&mut self, name: &str) -> Result<(), String> {
        let value = self
            .globals
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Variable '{}' non définie", name))?;

        self.push(value);
        Ok(())
    }
    //OP_SET_GLOBAL
    fn set_global(&mut self, name: &str) -> Result<(), String> {
        let value = self.peek().clone();

        if !self.globals.contains_key(name) {
            return Err(format!("Variable '{}' non définie", name));
        }

        self.globals.insert(name.to_string(), value);

        Ok(())
    }

    //OP_GET_LOCAL
    fn get_local(&mut self, slot: usize) {
        let value = self.stack[slot].clone();

        self.push(value);
    }

    //OP_SET_LOCAL
    fn set_local(&mut self, slot: usize) {
        let value = self.peek().clone();
        self.stack[slot] = value;
    }
}

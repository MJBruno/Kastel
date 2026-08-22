use std::collections::HashMap;
use std::rc::Rc;

use crate::chunk::OpCode;

use crate::compiler::Function;
use crate::error::RuntimeError;
use crate::value::ComparisonOp;
use crate::value::NumericOp;
use crate::value::Value;
use crate::value::print_value;

#[allow(dead_code)]
pub struct CallFrame {
    pub function: Rc<Function>,
    pub ip: usize,
    pub slot_start: usize,
}

pub struct VirtualMachine {
    pub stack: Vec<Value>,
    pub globals: HashMap<String, Value>,
    frames: Vec<CallFrame>,
}
#[allow(dead_code)]
impl VirtualMachine {
    pub fn new(function: Rc<Function>) -> Self {
        Self {
            stack: Vec::new(),
            globals: HashMap::new(),
            frames: vec![CallFrame {
                function,
                ip: 0,
                slot_start: 0,
            }],
        }
    }

    //Lire les instructions(bytecode) dans le chunk
    fn read_byte(&mut self) -> u8 {
        let frame = self.frames.last_mut().expect("No call frame");
        let byte = frame.function.chunk.code[frame.ip];
        frame.ip += 1;
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

    fn read_constant(&mut self) -> Value {
        let index = self.read_byte() as usize;
        let frame = self.current_frame();
        frame.function.chunk.constants[index].clone()
    }

    fn current_frame(&self) -> &CallFrame {
        self.frames.last().expect("No current CallFrame")
    }

    fn current_frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().expect("No current CallFrame")
    }

    fn current_ip(&self) -> usize {
        self.frames.last().expect("Aucun CallFrame").ip
    }

    pub fn run(&mut self) -> Result<(), RuntimeError> {
        loop {
            // self.print_stack();
            // let _ip = self.current_ip();

            // let (ip, chunk) = {
            //     let frame = self.current_frame();

            //     (frame.ip, frame.function.chunk.clone())
            // };

            // chunk.disassemble_instruction(ip);

            let instruction = self.read_byte();

            match instruction {
                x if x == OpCode::Constant.into() => {
                    let constant = self.read_constant();
                    self.push(constant);
                }

                x if x == OpCode::DefineGlobal.into() => {
                    let constant = self.read_constant();
                    let name = match constant {
                        Value::String(name) => name.clone(),
                        _ => return Err(RuntimeError::TypeError),
                    };
                    let value = self.pop();
                    self.globals.insert(name, value);
                }
                x if x == OpCode::GetGlobal.into() => {
                    let constant = self.read_constant();
                    let name = match constant {
                        Value::String(name) => name,
                        _ => return Err(RuntimeError::TypeError),
                    };

                    let value = match self.globals.get(&name) {
                        Some(value) => value.clone(),
                        None => {
                            return Err(RuntimeError::TypeError);
                        }
                    };

                    self.push(value);
                }

                x if x == OpCode::SetGlobal.into() => {
                    let constant = self.read_constant();
                    let name = match constant {
                        Value::String(name) => name.clone(),
                        _ => return Err(RuntimeError::TypeError),
                    };

                    if !self.globals.contains_key(&name) {
                        return Err(RuntimeError::TypeError);
                    }

                    let value = self.peek().clone();
                    self.globals.insert(name, value);
                }

                x if x == OpCode::GetLocal.into() => {
                    let slot = self.read_byte() as usize;
                    let frame = self.frames.last().expect("Aucun CallFrame");
                    let index = frame.slot_start + 1 + slot;
                    let value = self.stack[index].clone();

                    self.push(value);
                }

                x if x == OpCode::SetLocal.into() => {
                    let slot = self.read_byte() as usize;

                    let value = self.peek().clone();

                    let slot_start = self.current_frame().slot_start;

                    let index = slot_start + 1 + slot;

                    self.stack[index] = value;
                }

                x if x == OpCode::True.into() => {
                    self.push(Value::Boolean(true));
                }

                x if x == OpCode::False.into() => {
                    self.push(Value::Boolean(false));
                }
                x if x == OpCode::Nil.into() => {
                    self.push(Value::Nil);
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
                    let offset = self.read_short() as usize;

                    let frame = self.current_frame_mut();

                    frame.ip += offset;
                }
                x if x == OpCode::JumpIfFalse.into() => {
                    let offset = self.read_short() as usize;

                    if !self.peek().is_truthy() {
                        let frame = self.current_frame_mut();

                        frame.ip += offset;
                    }
                }
                x if x == OpCode::Loop.into() => {
                    let offset = self.read_short() as usize;

                    let frame = self.current_frame_mut();

                    frame.ip -= offset;
                }
                x if x == OpCode::Not.into() => {
                    let value = self.pop();
                    self.push(Value::Boolean(!value.is_truthy()));
                }
                x if x == OpCode::Call.into() => {
                    let arg_count = self.read_byte() as usize;

                    let callee_index = self.stack.len() - arg_count - 1;

                    let callee = self.stack[callee_index].clone();

                    match callee {
                        Value::Function(function) => {
                            self.call(function, arg_count)?;
                        }

                        _ => {
                            return Err(RuntimeError::NotCallable);
                        }
                    }
                }
                x if x == OpCode::Pop.into() => {
                    let _ = self.pop();
                }
                x if x == OpCode::Print.into() => {
                    let value = self.pop();
                    print_value(value);
                }

                x if x == OpCode::Return.into() => {
                    let result = self.pop();

                    let frame = self.frames.pop().expect("Aucun CallFrame");

                    self.stack.truncate(frame.slot_start);

                    if self.frames.is_empty() {
                        return Ok(());
                    }

                    self.push(result);
                }
                x if x == OpCode::Halt.into() => {
                    return Ok(());
                }
                _ => panic!("Unknown opcode: {instruction}"),
            }
        }
    }

    //OP_CALL
    fn call(&mut self, function: Rc<Function>, arg_count: usize) -> Result<(), RuntimeError> {
        if arg_count != function.arity {
            return Err(RuntimeError::WrongArgumentCount {
                expected: function.arity,
                found: arg_count,
            });
        }

        let callee_index = self.stack.len() - arg_count - 1;

        let frame = CallFrame {
            function,
            ip: 0,
            slot_start: callee_index,
        };

        self.frames.push(frame);

        Ok(())
    }
}

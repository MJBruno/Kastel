use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::bytecode::OpCode;
use crate::closure::Closure;
use crate::error::RuntimeError;
use crate::function::Function;
use crate::native::register_natives;
use crate::value::ComparisonOp;
use crate::value::NumericOp;
use crate::value::Value;
use crate::value::print_value;

#[derive(Debug)]
#[allow(dead_code)]
pub struct ObjUpvalue {
    pub slot: usize,
    pub closed: Option<Value>,
}
#[allow(dead_code)]
impl ObjUpvalue {
    pub fn new(slot: usize) -> Self {
        Self { slot, closed: None }
    }
}


#[allow(dead_code)]
pub struct CallFrame {
    pub closure: Rc<RefCell<Closure>>,
    pub ip: usize,
    pub slot_start: usize,
}
#[allow(dead_code)]
pub struct VirtualMachine {
    pub stack: Vec<Value>,
    pub globals: HashMap<String, Value>,
    frames: Vec<CallFrame>,
    open_upvalues: Vec<Rc<RefCell<ObjUpvalue>>>,
    natives: HashMap<String, Value>,
}
#[allow(dead_code)]
impl VirtualMachine {
    pub fn new(function: Rc<Function>) -> Self {
        let closure = Rc::new(RefCell::new(Closure {
            function,
            upvalues: Vec::new(),
        }));
        let mut vm = Self {
            stack: Vec::new(),
            globals: HashMap::new(),

            natives: HashMap::new(),
            frames: vec![CallFrame {
                closure,
                ip: 0,
                slot_start: 0,
            }],
            open_upvalues: Vec::new(),
        };

        register_natives(&mut vm.globals);

        vm
    }

    //Lire les instructions(bytecode) dans le chunk
    fn read_byte(&mut self) -> u8 {
        let frame = self.frames.last_mut().expect("No call frame");

        let byte = {
            let closure = frame.closure.borrow();

            closure.function.chunk.code[frame.ip]
        };

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

    // Affiche l'état de notre pile
    fn print_stack(&self) {
        print!("          ");
        for value in &self.stack {
            print!(" [ {} ]", value);
        }
        println!()
    }

    fn read_constant(&self, index: u8) -> Value {
        let frame = self.current_frame();

        frame.closure.borrow().function.chunk.constants[index as usize].clone()
    }

    fn read_constant_byte(&mut self) -> Value {
        let index = self.read_byte();
        self.read_constant(index)
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
            if cfg!(feature = "debug_trace") {
                self.debug_machine();
            }
            let instruction = self.read_byte();

            match instruction {
                x if x == OpCode::Constant.into() => {
                    let constant = self.read_constant_byte();
                    self.push(constant);
                }

                x if x == OpCode::DefineGlobal.into() => {
                    let constant = self.read_constant_byte();
                    let name = match constant {
                        Value::String(name) => name.clone(),
                        _ => return Err(RuntimeError::TypeError),
                    };
                    let value = self.pop();
                    self.globals.insert(name, value);
                }
                x if x == OpCode::GetGlobal.into() => {
                    let constant = self.read_constant_byte();
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
                    let constant = self.read_constant_byte();
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

                    let slot_start = self.frames.last().expect("Aucun CallFrame").slot_start;

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
                x if x == OpCode::Closure.into() => {
                    self.op_closure()?;
                }
                x if x == OpCode::SetUpvalue.into() => {
                    let index = self.read_byte() as usize;

                    self.set_upvalue(index)?;
                }
                x if x == OpCode::GetUpvalue.into() => {
                    let index = self.read_byte() as usize;

                    self.get_upvalue(index)?;
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
                        Value::Closure(function) => {
                            self.call(function, arg_count)?;
                        }

                        Value::NativeFunction(function) => {
                            let args_start = self.stack.len() - arg_count;

                            let args = self.stack[args_start..].to_vec();

                            let result = function(&args)?;

                            // Supprimer arguments + callee
                            self.stack.truncate(callee_index);

                            // Ajouter le résultat
                            self.push(result);
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

                    // Fermer les upvalues qui appartiennent
                    // à cette frame avant de supprimer ses locals.
                    self.close_upvalues(frame.slot_start);

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

    fn debug_machine(&mut self) {
        self.print_stack();
        let _ip = self.current_ip();
        let (ip, chunk) = {
            let frame = self.current_frame();
            let closure = frame.closure.borrow();
            (frame.ip, closure.function.chunk.clone())
        };
        chunk.disassemble_instruction(ip);
    }
    fn capture_upvalue(&mut self, slot: usize) -> Rc<RefCell<ObjUpvalue>> {
        let absolute_slot = self.current_frame().slot_start + 1 + slot;

        // Chercher un upvalue existant.
        for upvalue in &self.open_upvalues {
            if upvalue.borrow().slot == absolute_slot {
                return Rc::clone(upvalue);
            }
        }

        // Aucun upvalue existant.
        let upvalue = Rc::new(RefCell::new(ObjUpvalue {
            slot: absolute_slot,
            closed: None,
        }));

        self.open_upvalues.push(Rc::clone(&upvalue));

        upvalue
    }

    //OP_CALL
    fn call(
        &mut self,
        closure: Rc<RefCell<Closure>>,
        arg_count: usize,
    ) -> Result<(), RuntimeError> {
        let arity = closure.borrow().function.arity;

        if arg_count != arity {
            return Err(RuntimeError::WrongArgumentCount {
                expected: arity,
                found: arg_count,
            });
        }

        let callee_index = self.stack.len() - arg_count - 1;

        self.frames.push(CallFrame {
            closure,
            ip: 0,
            slot_start: callee_index,
        });

        Ok(())
    }

    fn op_closure(&mut self) -> Result<(), RuntimeError> {
        let constant_index = self.read_byte() as usize;

        let function = {
            let frame = self.current_frame();
            let closure = frame.closure.borrow();

            match closure
                .function
                .chunk
                .constants
                .get(constant_index)
                .cloned()
            {
                Some(Value::Function(function)) => function,

                _ => {
                    return Err(RuntimeError::InvalidFunction);
                }
            }
        };

        let mut closure = Closure {
            function: Rc::clone(&function),
            upvalues: Vec::with_capacity(function.upvalue_count),
        };

        for _ in 0..function.upvalue_count {
            let is_local = self.read_byte();
            let index = self.read_byte() as usize;

            let upvalue = if is_local != 0 {
                // Capture un local de la fonction courante.
                self.capture_upvalue(index)
            } else {
                // Récupère une upvalue déjà capturée par
                // la closure courante.
                let frame = self.current_frame();

                frame
                    .closure
                    .borrow()
                    .upvalues
                    .get(index)
                    .cloned()
                    .ok_or(RuntimeError::InvalidFunction)?
            };

            closure.upvalues.push(upvalue);
        }

        self.push(Value::Closure(Rc::new(RefCell::new(closure))));

        Ok(())
    }

    fn get_upvalue(&mut self, index: usize) -> Result<(), RuntimeError> {
        let upvalue = {
            let frame = self.current_frame();

            frame
                .closure
                .borrow()
                .upvalues
                .get(index)
                .cloned()
                .ok_or(RuntimeError::InvalidFunction)?
        };

        let value = {
            let upvalue = upvalue.borrow();

            match &upvalue.closed {
                Some(value) => value.clone(),

                None => self
                    .stack
                    .get(upvalue.slot)
                    .cloned()
                    .ok_or(RuntimeError::InvalidFunction)?,
            }
        };

        self.push(value);

        Ok(())
    }

    fn set_upvalue(&mut self, index: usize) -> Result<(), RuntimeError> {
        let upvalue = {
            let frame = self.current_frame();

            frame
                .closure
                .borrow()
                .upvalues
                .get(index)
                .cloned()
                .ok_or(RuntimeError::InvalidFunction)?
        };

        let value = self.peek().clone();

        let slot = {
            let mut upvalue_ref = upvalue.borrow_mut();

            if let Some(closed) = &mut upvalue_ref.closed {
                *closed = value;
                return Ok(());
            }

            upvalue_ref.slot
        };

        self.stack[slot] = value;

        Ok(())
    }
    fn close_upvalues(&mut self, last: usize) {
        let mut i = 0;

        while i < self.open_upvalues.len() {
            let slot = self.open_upvalues[i].borrow().slot;

            if slot >= last {
                let value = self.stack[slot].clone();

                {
                    let mut upvalue = self.open_upvalues[i].borrow_mut();
                    upvalue.closed = Some(value);
                }

                self.open_upvalues.remove(i);
            } else {
                i += 1;
            }
        }
    }

    // fn call_native(&mut self, function: NativeFn, arg_count: usize) -> Result<(), RuntimeError> {
    //     let callee_index = self.stack.len() - arg_count - 1;
    //     let args_start = callee_index + 1;
    //     let args = &self.stack[args_start..];
    //     let result = function(args)?;
    //     self.stack.truncate(callee_index);
    //     self.push(result);

    //     Ok(())
    // }
}

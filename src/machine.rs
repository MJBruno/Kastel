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

    // ============================================================
    // STACK
    // ============================================================

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().expect("Stack underflow")
    }

    fn peek(&self) -> &Value {
        self.stack.last().expect("Stack underflow")
    }

    // ============================================================
    // BYTECODE READER
    // ============================================================

    fn read_byte(&mut self) -> u8 {
        let frame = self.frames.last_mut().expect("No call frame");

        let byte = {
            let closure = frame.closure.borrow();

            closure.function.chunk.code[frame.ip]
        };

        frame.ip += 1;

        byte
    }

    fn read_short(&mut self) -> u16 {
        let high = self.read_byte() as u16;
        let low = self.read_byte() as u16;

        (high << 8) | low
    }

    fn read_constant(&self, index: u8) -> Value {
        let frame = self.current_frame();

        frame.closure.borrow().function.chunk.constants[index as usize].clone()
    }

    fn read_constant_byte(&mut self) -> Value {
        let index = self.read_byte();

        self.read_constant(index)
    }

    // ============================================================
    // FRAME
    // ============================================================

    fn current_frame(&self) -> &CallFrame {
        self.frames.last().expect("No current CallFrame")
    }

    fn current_frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().expect("No current CallFrame")
    }

    fn current_ip(&self) -> usize {
        self.frames.last().expect("Aucun CallFrame").ip
    }

    // ============================================================
    // DEBUG
    // ============================================================

    fn print_stack(&self) {
        print!("          ");

        for value in &self.stack {
            print!(" [ {} ]", value);
        }

        println!();
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

    // ============================================================
    // ARRAY
    // ============================================================

    fn op_array(&mut self, count: usize) -> Result<(), RuntimeError> {
        if self.stack.len() < count {
            return Err(RuntimeError::TypeError);
        }

        let start = self.stack.len() - count;

        let values = self.stack.drain(start..).collect::<Vec<_>>();

        self.push(Value::Array(Rc::new(RefCell::new(values))));

        Ok(())
    }

    fn array_index(value: Value) -> Result<usize, RuntimeError> {
        let index = match value {
            Value::Number(index) => index,

            _ => {
                return Err(RuntimeError::ArrayIndexNotInteger);
            }
        };

        if !index.is_finite() || index < 0.0 || index.fract() != 0.0 {
            return Err(RuntimeError::ArrayIndexNotInteger);
        }

        if index > usize::MAX as f64 {
            return Err(RuntimeError::ArrayIndexNotInteger);
        }

        Ok(index as usize)
    }

    fn op_get_index(&mut self) -> Result<(), RuntimeError> {
        let index_value = self.pop();
        let array_value = self.pop();

        let index = Self::array_index(index_value)?;

        let array = match array_value {
            Value::Array(array) => array,

            _ => {
                return Err(RuntimeError::TypeError);
            }
        };

        let array_ref = array.borrow();

        let value = array_ref
            .get(index)
            .cloned()
            .ok_or(RuntimeError::ArrayIndexOutOfBounds {
                index,
                length: array_ref.len(),
            })?;

        self.push(value);

        Ok(())
    }

    fn op_set_index(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop();
        let index_value = self.pop();
        let array_value = self.pop();

        let index = Self::array_index(index_value)?;

        let array = match array_value {
            Value::Array(array) => array,

            _ => {
                return Err(RuntimeError::TypeError);
            }
        };

        {
            let mut array_ref = array.borrow_mut();

            let length = array_ref.len();

            let slot = array_ref
                .get_mut(index)
                .ok_or(RuntimeError::ArrayIndexOutOfBounds { index, length })?;

            *slot = value;
        }

        self.push(Value::Array(array));

        Ok(())
    }

    // ============================================================
    // VM
    // ============================================================

    pub fn run(&mut self) -> Result<(), RuntimeError> {
        loop {
            if cfg!(feature = "debug_trace") {
                self.debug_machine();
            }

            let instruction = self.read_byte();

            match instruction {
                // =================================================
                // CONSTANTS
                // =================================================
                x if x == OpCode::Constant.into() => {
                    let constant = self.read_constant_byte();

                    self.push(constant);
                }

                x if x == OpCode::DefineGlobal.into() => {
                    let constant = self.read_constant_byte();

                    let name = match constant {
                        Value::String(name) => name,

                        _ => {
                            return Err(RuntimeError::TypeError);
                        }
                    };

                    let value = self.pop();

                    self.globals.insert(name, value);
                }

                x if x == OpCode::GetGlobal.into() => {
                    let constant = self.read_constant_byte();

                    let name = match constant {
                        Value::String(name) => name,

                        _ => {
                            return Err(RuntimeError::TypeError);
                        }
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
                        Value::String(name) => name,

                        _ => {
                            return Err(RuntimeError::TypeError);
                        }
                    };

                    if !self.globals.contains_key(&name) {
                        return Err(RuntimeError::TypeError);
                    }

                    let value = self.peek().clone();

                    self.globals.insert(name, value);
                }

                // =================================================
                // LOCALS
                // =================================================
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

                // =================================================
                // LITERALS
                // =================================================
                x if x == OpCode::True.into() => {
                    self.push(Value::Boolean(true));
                }

                x if x == OpCode::False.into() => {
                    self.push(Value::Boolean(false));
                }

                x if x == OpCode::Nil.into() => {
                    self.push(Value::Nil);
                }

                // =================================================
                // ARITHMETIC
                // =================================================
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

                x if x == OpCode::Negate.into() => {
                    let value = self.pop();

                    let result = Value::negate_values(value).expect("Opérand must be value");

                    self.push(result);
                }

                // =================================================
                // COMPARISON
                // =================================================
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

                x if x == OpCode::Not.into() => {
                    let value = self.pop();

                    self.push(Value::Boolean(!value.is_truthy()));
                }

                // =================================================
                // ARRAYS
                // =================================================
                x if x == OpCode::Array.into() => {
                    let count = self.read_byte() as usize;

                    self.op_array(count)?;
                }

                x if x == OpCode::GetIndex.into() => {
                    self.op_get_index()?;
                }

                x if x == OpCode::SetIndex.into() => {
                    self.op_set_index()?;
                }

                // =================================================
                // CLOSURES
                // =================================================
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

                // =================================================
                // CONTROL FLOW
                // =================================================
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

                // =================================================
                // CALL
                // =================================================
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

                            // Supprimer arguments + callee.
                            self.stack.truncate(callee_index);

                            // Ajouter le résultat.
                            self.push(result);
                        }

                        _ => {
                            return Err(RuntimeError::NotCallable);
                        }
                    }
                }

                // =================================================
                // STACK
                // =================================================
                x if x == OpCode::Pop.into() => {
                    let _ = self.pop();
                }

                // =================================================
                // PRINT
                // =================================================
                x if x == OpCode::Print.into() => {
                    let value = self.pop();

                    print_value(value);
                }

                // =================================================
                // RETURN
                // =================================================
                x if x == OpCode::Return.into() => {
                    let result = self.pop();

                    let frame = self.frames.pop().expect("Aucun CallFrame");

                    self.close_upvalues(frame.slot_start);

                    self.stack.truncate(frame.slot_start);

                    if self.frames.is_empty() {
                        return Ok(());
                    }

                    self.push(result);
                }

                // =================================================
                // HALT
                // =================================================
                x if x == OpCode::Halt.into() => {
                    return Ok(());
                }

                _ => {
                    panic!("Unknown opcode: {instruction}");
                }
            }
        }
    }

    // ============================================================
    // CALL
    // ============================================================

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

    // ============================================================
    // CLOSURE
    // ============================================================

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
                self.capture_upvalue(index)
            } else {
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

    // ============================================================
    // UPVALUES
    // ============================================================

    fn capture_upvalue(&mut self, slot: usize) -> Rc<RefCell<ObjUpvalue>> {
        let absolute_slot = self.current_frame().slot_start + 1 + slot;

        for upvalue in &self.open_upvalues {
            if upvalue.borrow().slot == absolute_slot {
                return Rc::clone(upvalue);
            }
        }

        let upvalue = Rc::new(RefCell::new(ObjUpvalue {
            slot: absolute_slot,
            closed: None,
        }));

        self.open_upvalues.push(Rc::clone(&upvalue));

        upvalue
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
}

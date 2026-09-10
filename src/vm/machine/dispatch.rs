use std::collections::HashMap;

use super::VirtualMachine;

use crate::{
    bytecode::chunk::OpCode,
    error::runtime_error::RuntimeError,
    runtime::{
        object::Object,
        value::{ComparisonOp, NumericOp, Value},
    },
};

impl VirtualMachine {
    pub(crate) fn dispatch(&mut self, instruction: u8) -> Result<bool, RuntimeError> {
        let opcode =
            OpCode::try_from(instruction).map_err(|_| RuntimeError::InvalidOpcode(instruction))?;

        match opcode {
            // ========================================================
            // CONSTANTS / GLOBALS
            // ========================================================
            OpCode::Constant => {
                let constant = self.read_constant_byte()?;

                self.push(constant);
            }

            OpCode::DefineGlobal => {
                self.define_global()?;
            }

            OpCode::GetGlobal => {
                self.get_global()?;
            }

            OpCode::SetGlobal => {
                self.set_global()?;
            }

            OpCode::GetLocal => {
                self.get_local()?;
            }

            OpCode::SetLocal => {
                self.set_local()?;
            }

            OpCode::GetUpvalue => {
                let index = self.read_byte()? as usize;

                self.get_upvalue(index)?;
            }

            OpCode::SetUpvalue => {
                let index = self.read_byte()? as usize;

                self.set_upvalue(index)?;
            }

            // ========================================================
            // LITERALS
            // ========================================================
            OpCode::True => {
                self.push(Value::Boolean(true));
            }

            OpCode::False => {
                self.push(Value::Boolean(false));
            }

            OpCode::Nil => {
                self.push(Value::Nil);
            }

            // ========================================================
            // ARITHMETIC
            // ========================================================
            OpCode::Add => {
                self.add()?;
            }

            OpCode::Subtract => {
                self.numeric_binary(NumericOp::Subtract)?;
            }

            OpCode::Multiply => {
                self.numeric_binary(NumericOp::Multiply)?;
            }

            OpCode::Divide => {
                self.numeric_binary(NumericOp::Divide)?;
            }

            OpCode::Modulo => {
                self.numeric_binary(NumericOp::Modulo)?;
            }

            OpCode::Negate => {
                self.negate()?;
            }

            // ========================================================
            // BITWISE
            // ========================================================
            OpCode::BitAnd => {
                self.bitwise_binary(0)?;
            }

            OpCode::BitOr => {
                self.bitwise_binary(1)?;
            }

            OpCode::BitXor => {
                self.bitwise_binary(2)?;
            }

            OpCode::BitNot => {
                self.bitwise_not()?;
            }

            OpCode::ShiftLeft => {
                self.shift(true)?;
            }

            OpCode::ShiftRight => {
                self.shift(false)?;
            }

            // ========================================================
            // COMPARISON
            // ========================================================
            OpCode::Equal => {
                let b = self.pop()?;
                let a = self.pop()?;

                self.push(Value::Boolean(Value::equals(a, b)));
            }

            OpCode::Greater => {
                self.compare(ComparisonOp::Greater)?;
            }

            OpCode::Less => {
                self.compare(ComparisonOp::Less)?;
            }

            OpCode::Not => {
                self.not()?;
            }

            // ========================================================
            // OBJECT / PROPERTY
            // ========================================================
            OpCode::GetProperty => {
                self.get_property()?;
            }

            OpCode::SetProperty => {
                self.set_property()?;
            }

            // ========================================================
            // ARRAY
            // ========================================================
            OpCode::Array => {
                let count = self.read_byte()? as usize;

                self.op_array(count)?;
            }

            OpCode::Object => {
                let pair_count = self.read_byte()? as usize;

                self.op_object(pair_count)?;
            }

            OpCode::GetIndex => {
                self.op_get_index()?;
            }

            OpCode::SetIndex => {
                self.op_set_index()?;
            }

            OpCode::ArrayLength => {
                self.op_array_length()?;
            }

            OpCode::ArrayPush => {
                self.op_array_push()?;
            }

            OpCode::ArrayPop => {
                self.op_array_pop()?;
            }

            OpCode::ArrayInsert => {
                self.op_array_insert()?;
            }

            OpCode::ArrayRemove => {
                self.op_array_remove()?;
            }

            OpCode::ArrayClear => {
                self.op_array_clear()?;
            }

            OpCode::ArrayContains => {
                self.op_array_contains()?;
            }

            // ========================================================
            // ITERATORS
            // ========================================================
            OpCode::GetIterator => {
                self.op_get_iterator()?;
            }

            OpCode::IteratorHasNext => {
                self.op_iterator_has_next()?;
            }

            OpCode::IteratorNext => {
                self.op_iterator_next()?;
            }

            // ========================================================
            // FUNCTIONS / CLOSURES
            // ========================================================
            OpCode::Closure => {
                self.op_closure()?;
            }

            OpCode::Call => {
                let arg_count = self.read_byte()? as usize;

                self.execute_call(arg_count)?;
            }

            OpCode::InvokeMethod => {
                let method_constant = self.read_byte()? as usize;
                let arg_count = self.read_byte()? as usize;

                self.op_invoke_method(method_constant, arg_count)?;
            }
            OpCode::Class => {
                let method_count = self.read_byte()? as usize;

                self.op_class(method_count)?;
            }

            OpCode::NewInstance => {
                let arg_count = self.read_byte()? as usize;

                self.op_new_instance(arg_count)?;
            }
            // ========================================================
            // MODULES
            // ========================================================
            OpCode::Import => {
                self.import_module()?;
            }

            // ========================================================
            // CONTROL FLOW
            // ========================================================
            OpCode::Jump => {
                self.jump()?;
            }

            OpCode::JumpIfFalse => {
                self.jump_if_false()?;
            }

            OpCode::Loop => {
                self.loop_back()?;
            }

            OpCode::Pop => {
                self.pop()?;
            }

            // ========================================================
            // RETURN / HALT
            // ========================================================
            OpCode::Return => {
                self.execute_return()?;

                return Ok(self.frames.is_empty());
            }

            OpCode::Halt => {
                return Ok(true);
            }

            // ========================================================
            // EXCEPTIONS
            // ========================================================
            OpCode::PushExceptionHandler => {
                let catch_raw = self.read_short()?;
                let finally_raw = self.read_short()?;

                let catch_ip = if catch_raw == u16::MAX {
                    None
                } else {
                    Some(catch_raw as usize)
                };

                let finally_ip = if finally_raw == u16::MAX {
                    None
                } else {
                    Some(finally_raw as usize)
                };

                self.register_exception_handler(catch_ip, finally_ip)?;
            }

            OpCode::PopExceptionHandler => {
                self.unregister_exception_handler()?;
            }

            OpCode::Throw => {
                let value = self.pop()?;

                return Err(RuntimeError::Thrown(value));
            }
            #[allow(clippy::single_match)]
            OpCode::FinallyEnd => match self.pending_exception.take() {
                Some(pending) => match pending.rethrow {
                    true => return Err(RuntimeError::Thrown(pending.value)),
                    false => (),
                },
                _ => (),
            },
        }

        Ok(false)
    }

    pub(crate) fn op_class(&mut self, method_count: usize) -> Result<(), RuntimeError> {
        let method_values = method_count
            .checked_mul(2)
            .ok_or(RuntimeError::InvalidFunction)?;

        let total = method_values
            .checked_add(2)
            .ok_or(RuntimeError::InvalidFunction)?;

        if self.stack.len() < total {
            return Err(RuntimeError::StackUnderflow);
        }

        let start = self.stack.len() - total;

        // ========================================================
        // SUPERCLASS
        // ========================================================

        let superclass_value = self.stack[start].clone();

        let superclass = match superclass_value {
            Value::Nil => None,

            Value::Object(handle) => {
                if !matches!(&*handle.borrow(), Object::Class { .. }) {
                    return Err(RuntimeError::TypeError);
                }

                Some(handle)
            }

            _ => {
                return Err(RuntimeError::TypeError);
            }
        };

        // ========================================================
        // NAME
        // ========================================================

        let class_name = self.stack[start + 1]
            .as_string_value()
            .ok_or(RuntimeError::TypeError)?;

        // ========================================================
        // METHODS
        // ========================================================

        let mut methods = HashMap::with_capacity(method_count);

        for index in 0..method_count {
            let base = start + 2 + index * 2;

            let method_name = self.stack[base]
                .as_string_value()
                .ok_or(RuntimeError::TypeError)?;

            let method = self.stack[base + 1].clone();

            if !matches!(
                &method,
                Value::Object(handle)
                    if matches!(
                        &*handle.borrow(),
                        Object::Closure(_)
                    )
            ) {
                return Err(RuntimeError::NotCallable);
            }

            methods.insert(method_name, method);
        }

        self.stack.truncate(start);

        self.push(Value::new_class(class_name, superclass, methods));

        Ok(())
    }

    pub(crate) fn op_new_instance(&mut self, arg_count: usize) -> Result<(), RuntimeError> {
        let required = arg_count
            .checked_add(1)
            .ok_or(RuntimeError::InvalidFunction)?;

        if self.stack.len() < required {
            return Err(RuntimeError::StackUnderflow);
        }

        let class_index = self.stack.len() - required;

        let class_value = self.stack[class_index].clone();

        let class_handle = match &class_value {
            Value::Object(handle) => match &*handle.borrow() {
                Object::Class { .. } => handle.clone(),

                _ => {
                    return Err(RuntimeError::NotCallable);
                }
            },

            _ => {
                return Err(RuntimeError::NotCallable);
            }
        };

        let instance = Value::new_instance(class_handle.clone());

        let args = self.stack[class_index + 1..].to_vec();

        self.stack.truncate(class_index);

        // L'instance reste une racine GC pendant l'appel
        // du constructeur.
        self.push(instance.clone());

        let init = {
            let class = class_handle.borrow();

            match &*class {
                Object::Class { methods, .. } => methods.get("init").cloned(),

                _ => None,
            }
        };

        match init {
            Some(init) => {
                let mut init_args = Vec::with_capacity(arg_count + 1);

                init_args.push(instance.clone());
                init_args.extend(args);

                self.invoke_sync(init, &init_args)?;

                self.pop()?;
            }

            None => {
                if !args.is_empty() {
                    return Err(RuntimeError::WrongArgumentCount {
                        expected: 0,
                        found: args.len(),
                    });
                }

                self.pop()?;
            }
        }

        self.push(instance);

        Ok(())
    }
}

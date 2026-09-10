use super::VirtualMachine;
use std::collections::HashMap;

use crate::{
    bytecode::chunk::OpCode,
    error::runtime_error::RuntimeError,
    runtime::{
        gc_handle::Gc,
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
            OpCode::InvokeBaseMethod => {
                let method_constant = self.read_byte()? as usize;
                let arg_count = self.read_byte()? as usize;

                self.op_invoke_base_method(method_constant, arg_count)?;
            }
            OpCode::Interface => {
                let method_count = self.read_byte()? as usize;

                self.op_interface(method_count)?;
            }

            OpCode::Class => {
                let base_count = self.read_byte()? as usize;
                let method_count = self.read_byte()? as usize;

                self.op_class(base_count, method_count)?;
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

    pub(crate) fn op_class(
        &mut self,
        base_count: usize,
        method_count: usize,
    ) -> Result<(), RuntimeError> {
        let method_values = method_count
            .checked_mul(2)
            .ok_or(RuntimeError::InvalidFunction)?;

        let total = base_count
            .checked_add(1)
            .and_then(|value| value.checked_add(method_values))
            .ok_or(RuntimeError::InvalidFunction)?;

        if self.stack.len() < total {
            return Err(RuntimeError::StackUnderflow);
        }

        let start = self.stack.len() - total;

        // ========================================================
        // BASES
        // ========================================================

        let mut superclass = None;
        let mut interfaces = Vec::new();

        for index in 0..base_count {
            let value = self.stack[start + index].clone();

            let handle = match value {
                Value::Object(handle) => handle,

                _ => {
                    return Err(RuntimeError::TypeError);
                }
            };

            match &*handle.clone().borrow() {
                Object::Class { .. } => {
                    // Une seule classe parente autorisée.
                    if superclass.is_some() {
                        return Err(RuntimeError::TypeError);
                    }

                    superclass = Some(handle);
                }

                Object::Interface { .. } => {
                    interfaces.push(handle);
                }

                _ => {
                    return Err(RuntimeError::TypeError);
                }
            }
        }

        // ========================================================
        // CLASS NAME
        // ========================================================

        let class_name = self.stack[start + base_count]
            .as_string_value()
            .ok_or(RuntimeError::TypeError)?;

        // ========================================================
        // METHODS
        // ========================================================

        let methods_start = start + base_count + 1;

        let mut methods = HashMap::with_capacity(method_count);

        for index in 0..method_count {
            let base = methods_start + index * 2;

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

            if methods.insert(method_name, method).is_some() {
                return Err(RuntimeError::TypeError);
            }
        }

        // ========================================================
        // CREATE CLASS
        // ========================================================

        self.stack.truncate(start);

        let class_value = Value::new_class(class_name, superclass, interfaces, methods);

        let class_handle = match &class_value {
            Value::Object(handle) => handle.clone(),

            _ => {
                return Err(RuntimeError::InvalidFunction);
            }
        };

        // ========================================================
        // OWNER CLASS DES MÉTHODES
        // ========================================================
        //
        // Nécessaire pour :
        //
        //     base.foo()
        //
        // Une méthode doit savoir quelle classe la possède.

        {
            let mut class_object = class_handle.borrow_mut();

            let Object::Class { methods, .. } = &mut *class_object else {
                return Err(RuntimeError::InvalidFunction);
            };

            for method in methods.values() {
                if let Value::Object(handle) = method {
                    let mut object = handle.borrow_mut();

                    if let Object::Closure(closure) = &mut *object {
                        closure.owner_class = Some(class_handle.clone());
                    }
                }
            }
        }

        // ========================================================
        // INTERFACE VALIDATION
        // ========================================================

        Self::validate_interfaces(&class_handle)?;

        // ========================================================
        // PUSH RESULT
        // ========================================================

        self.push(class_value);

        Ok(())
    }

    fn find_class_method_from(class: Gc<Object>, name: &str) -> Option<Value> {
        let mut current = Some(class);

        while let Some(handle) = current {
            let object = handle.borrow();

            match &*object {
                Object::Class {
                    superclass,
                    methods,
                    ..
                } => {
                    if let Some(method) = methods.get(name) {
                        return Some(method.clone());
                    }

                    current = superclass.clone();
                }

                _ => return None,
            }
        }

        None
    }

    fn validate_interfaces(class: &Gc<Object>) -> Result<(), RuntimeError> {
        let interfaces = {
            let object = class.borrow();

            match &*object {
                Object::Class { interfaces, .. } => interfaces.clone(),

                _ => return Err(RuntimeError::TypeError),
            }
        };

        for interface in interfaces {
            let requirements = {
                let object = interface.borrow();

                match &*object {
                    Object::Interface { methods, .. } => methods.clone(),

                    _ => return Err(RuntimeError::TypeError),
                }
            };

            for (name, required_arity) in requirements {
                let method = Self::find_class_method_from(class.clone(), &name);

                let Some(method) = method else {
                    return Err(RuntimeError::TypeError);
                };

                let actual_arity = match method {
                    Value::Object(handle) => {
                        let object = handle.borrow();

                        match &*object {
                            Object::Closure(closure) => closure
                                .function
                                .arity
                                .checked_sub(1)
                                .ok_or(RuntimeError::TypeError)?,

                            _ => return Err(RuntimeError::TypeError),
                        }
                    }

                    _ => return Err(RuntimeError::TypeError),
                };

                if actual_arity != required_arity {
                    return Err(RuntimeError::WrongArgumentCount {
                        expected: required_arity,
                        found: actual_arity,
                    });
                }
            }
        }

        Ok(())
    }
    pub(crate) fn op_interface(&mut self, method_count: usize) -> Result<(), RuntimeError> {
        let total = method_count
            .checked_mul(2)
            .and_then(|value| value.checked_add(1))
            .ok_or(RuntimeError::InvalidFunction)?;

        if self.stack.len() < total {
            return Err(RuntimeError::StackUnderflow);
        }

        let start = self.stack.len() - total;

        let name = self.stack[start]
            .as_string_value()
            .ok_or(RuntimeError::TypeError)?;

        let mut methods = HashMap::with_capacity(method_count);

        for index in 0..method_count {
            let base = start + 1 + index * 2;

            let method_name = self.stack[base]
                .as_string_value()
                .ok_or(RuntimeError::TypeError)?;

            let arity = match &self.stack[base + 1] {
                Value::Integer(value) if *value >= 0 => {
                    usize::try_from(*value).map_err(|_| RuntimeError::TypeError)?
                }

                _ => return Err(RuntimeError::TypeError),
            };

            if methods.insert(method_name.clone(), arity).is_some() {
                return Err(RuntimeError::TypeError);
            }
        }

        self.stack.truncate(start);

        self.push(Value::new_interface(name, methods));

        Ok(())
    }

    fn find_base_method(class: Gc<Object>, name: &str) -> Option<Value> {
        let parent = {
            let object = class.borrow();

            match &*object {
                Object::Class { superclass, .. } => superclass.clone(),

                _ => None,
            }
        };

        let mut current = parent;

        while let Some(handle) = current {
            let object = handle.borrow();

            match &*object {
                Object::Class {
                    superclass,
                    methods,
                    ..
                } => {
                    if let Some(method) = methods.get(name) {
                        return Some(method.clone());
                    }

                    current = superclass.clone();
                }

                _ => return None,
            }
        }

        None
    }
    pub(crate) fn op_invoke_base_method(
        &mut self,
        method_constant: usize,
        arg_count: usize,
    ) -> Result<(), RuntimeError> {
        let method_value = self.read_constant(method_constant.try_into().unwrap())?;

        let method_name = method_value
            .as_string_value()
            .ok_or(RuntimeError::TypeError)?;

        let frame = self
            .frames
            .last()
            .ok_or(RuntimeError::InvalidFunction)?
            .clone();

        let this_index = frame
            .slot_start
            .checked_add(1)
            .ok_or(RuntimeError::InvalidFunction)?;

        if this_index >= self.stack.len() {
            return Err(RuntimeError::StackUnderflow);
        }

        let this_value = self.stack[this_index].clone();

        // Les derniers éléments sont les arguments de base.speak(...).
        if self.stack.len() < arg_count {
            return Err(RuntimeError::StackUnderflow);
        }

        let args_start = self.stack.len() - arg_count;

        let args = self.stack[args_start..].to_vec();

        self.stack.truncate(args_start);

        let owner_class = {
            let closure = super::bytecode::frame_closure(&frame.closure);
            closure.owner_class.clone().ok_or(RuntimeError::TypeError)?
        };

        let method = Self::find_base_method(owner_class, &method_name).ok_or(
            RuntimeError::ObjectFieldNotFound {
                name: method_name,
                suggestion: None,
            },
        )?;

        // Nouvel appel :
        //
        // [method, this, arg1, arg2, ...]
        self.push(method);
        self.push(this_value);

        for argument in args {
            self.push(argument);
        }

        self.execute_call(arg_count + 1)?;

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

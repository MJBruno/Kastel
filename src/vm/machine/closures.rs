 
use std::rc::Rc;

use super::VirtualMachine;
use super::bytecode::frame_closure;
use crate::error::runtime_error::RuntimeError;
use crate::runtime::object::Object;
use crate::runtime::value::Value;

impl VirtualMachine {
    pub(crate) fn op_closure(&mut self) -> Result<(), RuntimeError> {
        let constant_index = self.read_byte()? as usize;

        let function = {
            let frame = self.current_frame()?;
            let closure = frame_closure(&frame.closure);

            match closure.function.chunk.constants.get(constant_index).cloned() {
                Some(Value::Object(handle)) => {
                    match &*handle.borrow() {
                        Object::Function(function) => Rc::clone(function),
                        _ => return Err(RuntimeError::InvalidFunction),
                    }
                }
                _ => return Err(RuntimeError::InvalidFunction),
            }
        };

        let mut pending_upvalues = Vec::with_capacity(function.upvalue_count);

        for _ in 0..function.upvalue_count {
            let is_local = self.read_byte()?;
            let index = self.read_byte()? as usize;

            let upvalue = if is_local != 0 {
                self.capture_upvalue(index)
            } else {
                let frame = self.current_frame()?;
                frame_closure(&frame.closure)
                    .upvalues
                    .get(index)
                    .cloned()
                    .ok_or(RuntimeError::InvalidFunction)?
            };

            pending_upvalues.push(upvalue);
        }

        let closure = Object::new_closure(
            Rc::clone(&function),
            pending_upvalues,
        );

        self.push(Value::Object(closure));

        Ok(())
    }

   

    pub(crate) fn get_upvalue(
        &mut self,
        index: usize,
    ) -> Result<(), RuntimeError> {
        let upvalue = {
            let frame = self.current_frame()?;

            frame_closure(&frame.closure)
                .upvalues
                .get(index)
                .cloned()
                .ok_or(RuntimeError::InvalidFunction)?
        };

        let value = {
            let upvalue_ref = upvalue.borrow();

            match &upvalue_ref.closed {
                Some(value) => value.clone(),

                None => self
                    .stack
                    .get(upvalue_ref.slot)
                    .cloned()
                    .ok_or(RuntimeError::InvalidFunction)?,
            }
        };

        self.push(value);

        Ok(())
    }

    pub(crate) fn set_upvalue(
        &mut self,
        index: usize,
    ) -> Result<(), RuntimeError> {
        let upvalue = {
            let frame = self.current_frame()?;

            frame_closure(&frame.closure)
                .upvalues
                .get(index)
                .cloned()
                .ok_or(RuntimeError::InvalidFunction)?
        };

        let value = self.peek()?.clone();

        let slot = {
            let mut upvalue_ref = upvalue.borrow_mut();

            if let Some(closed) = &mut upvalue_ref.closed {
                *closed = value;
                return Ok(());
            }

            upvalue_ref.slot
        };

        if slot >= self.stack.len() {
            return Err(RuntimeError::InvalidFunction);
        }

        self.stack[slot] = value;

        Ok(())
    }

   
}
 

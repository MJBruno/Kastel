use std::cell::RefCell;
use std::rc::Rc;

use super::VirtualMachine;
use super::bytecode::frame_closure;
use crate::error::runtime_error::RuntimeError;
use crate::runtime::gc;
use crate::runtime::object::Object;
use crate::runtime::upvalue::ObjUpvalue;
use crate::runtime::value::Value;

impl VirtualMachine {
    pub(crate) fn op_closure(&mut self) -> Result<(), RuntimeError> {
        let constant_index = self.read_byte()? as usize;
        let function = {
            let frame = self.current_frame()?;
            let closure = frame_closure(&frame.closure);
            match closure.function.chunk.constants.get(constant_index).cloned() {
                Some(Value::Object(handle)) => match &*handle.borrow() {
                    Object::Function(function) => Rc::clone(function),
                    _ => return Err(RuntimeError::InvalidFunction),
                },
                _ => return Err(RuntimeError::InvalidFunction),
            }
        };

        let mut pending_upvalues = Vec::with_capacity(function.upvalue_count);
        for _ in 0..function.upvalue_count {
            let is_local = self.read_byte()?;
            let index = self.read_byte()? as usize;
            let upvalue = if is_local != 0 {
                self.capture_upvalue(index)?
            } else {
                let frame = self.current_frame()?;
                frame_closure(&frame.closure).upvalues.get(index).cloned().ok_or(RuntimeError::InvalidFunction)?
            };
            pending_upvalues.push(upvalue);
        }

        let closure = Object::new_closure(Rc::clone(&function), pending_upvalues);
        self.push(Value::Object(closure));
        Ok(())
    }

    pub(crate) fn capture_upvalue(&mut self, slot: usize) -> Result<Rc<RefCell<ObjUpvalue>>, RuntimeError> {
        let (slot_start, local_count) = {
            let frame = self.current_frame()?;
            let closure = frame_closure(&frame.closure);
            (frame.slot_start, closure.function.local_count as usize)
        };

        if slot >= local_count { return Err(RuntimeError::InvalidFunction); }
        let absolute_slot = slot_start.checked_add(1).and_then(|i| i.checked_add(slot)).ok_or(RuntimeError::InvalidFunction)?;
        if absolute_slot >= self.stack.len() { return Err(RuntimeError::InvalidFunction); }

        for upvalue in &self.open_upvalues {
            if upvalue.borrow().slot == absolute_slot { return Ok(Rc::clone(upvalue)); }
        }

        let upvalue = Rc::new(RefCell::new(ObjUpvalue::new(absolute_slot)));
        gc::register_upvalue(&upvalue);
        self.open_upvalues.push(Rc::clone(&upvalue));
        Ok(upvalue)
    }

    pub(crate) fn get_upvalue(&mut self, index: usize) -> Result<(), RuntimeError> {
        let upvalue = {
            let frame = self.current_frame()?;
            frame_closure(&frame.closure).upvalues.get(index).cloned().ok_or(RuntimeError::InvalidFunction)?
        };
        let value = {
            let upvalue_ref = upvalue.borrow();
            match &upvalue_ref.closed {
                Some(value) => value.clone(),
                None => self.stack.get(upvalue_ref.slot).cloned().ok_or(RuntimeError::InvalidFunction)?,
            }
        };
        self.push(value);
        Ok(())
    }

    pub(crate) fn set_upvalue(&mut self, index: usize) -> Result<(), RuntimeError> {
        let upvalue = {
            let frame = self.current_frame()?;
            frame_closure(&frame.closure).upvalues.get(index).cloned().ok_or(RuntimeError::InvalidFunction)?
        };
        let value = self.peek()?.clone();
        let slot = {
            let mut upvalue_ref = upvalue.borrow_mut();
            if let Some(closed) = &mut upvalue_ref.closed { *closed = value; return Ok(()); }
            upvalue_ref.slot
        };
        if slot >= self.stack.len() { return Err(RuntimeError::InvalidFunction); }
        self.stack[slot] = value;
        Ok(())
    }

    pub(crate) fn close_upvalues(&mut self, last: usize) -> Result<(), RuntimeError> {
        let mut i = 0;
        while i < self.open_upvalues.len() {
            let slot = self.open_upvalues[i].borrow().slot;
            if slot >= last {
                let value = self.stack.get(slot).cloned().ok_or(RuntimeError::InvalidFunction)?;
                self.open_upvalues[i].borrow_mut().closed = Some(value);
                self.open_upvalues.remove(i);
            } else { i += 1; }
        }
        Ok(())
    }
}

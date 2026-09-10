use std::cell::Ref;

use super::{CallFrame, VirtualMachine};
use crate::error::runtime_error::RuntimeError;
use crate::runtime::closure::Closure;
use crate::runtime::gc_handle::Gc;
use crate::runtime::object::Object;
use crate::runtime::value::Value;

pub(crate) fn frame_closure(handle: &Gc<Object>) -> Ref<'_, Closure> {
    Ref::map(handle.borrow(), |object| match object {
        Object::Closure(closure) => closure,
        _ => unreachable!("CallFrame.closure est toujours un Object::Closure"),
    })
}

impl VirtualMachine {
    pub(crate) fn current_frame(&self) -> Result<&CallFrame, RuntimeError> {
        self.frames.last().ok_or(RuntimeError::InvalidFunction)
    }

    pub(crate) fn current_frame_mut(&mut self) -> Result<&mut CallFrame, RuntimeError> {
        self.frames.last_mut().ok_or(RuntimeError::InvalidFunction)
    }

    pub(crate) fn read_byte(&mut self) -> Result<u8, RuntimeError> {
        let frame = self
            .frames
            .last_mut()
            .ok_or(RuntimeError::InvalidFunction)?;

        let byte = {
            let closure = frame_closure(&frame.closure);

            closure
                .function
                .chunk
                .code
                .get(frame.ip)
                .copied()
                .ok_or(RuntimeError::InvalidFunction)?
        };

        frame.ip = frame
            .ip
            .checked_add(1)
            .ok_or(RuntimeError::InvalidFunction)?;

        Ok(byte)
    }
    pub(crate) fn read_short(&mut self) -> Result<u16, RuntimeError> {
        let high = self.read_byte()? as u16;
        let low = self.read_byte()? as u16;
        Ok((high << 8) | low)
    }

    pub(crate) fn read_constant(&self, index: u8) -> Result<Value, RuntimeError> {
        let frame = self.current_frame()?;
        frame_closure(&frame.closure)
            .function
            .chunk
            .constants
            .get(index as usize)
            .cloned()
            .ok_or(RuntimeError::InvalidFunction)
    }

    pub(crate) fn read_constant_byte(&mut self) -> Result<Value, RuntimeError> {
        let index = self.read_byte()?;
        self.read_constant(index)
    }

    pub(crate) fn current_position(&self) -> Result<(usize, usize), RuntimeError> {
        let frame = self.current_frame()?;
        let closure = frame_closure(&frame.closure);
        Ok(closure.function.chunk.position_at(frame.ip))
    }
}

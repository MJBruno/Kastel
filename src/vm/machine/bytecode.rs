use std::cell::Ref;

use super::{CallFrame, VirtualMachine};

use crate::{
    error::runtime_error::RuntimeError, runtime::closure::Closure, runtime::gc_handle::Gc,
    runtime::object::Object, runtime::value::Value,
};

pub(crate) fn frame_closure(handle: &Gc<Object>) -> Ref<'_, Closure> {
    Ref::map(handle.borrow(), |object| match object {
        Object::Closure(closure) => closure,

        _ => unreachable!("CallFrame.closure est toujours un Object::Closure"),
    })
}

impl VirtualMachine {
    #[inline]
    pub(crate) fn current_frame(&self) -> Result<&CallFrame, RuntimeError> {
        self.frames.last().ok_or(RuntimeError::InvalidFunction)
    }

    #[inline]
    pub(crate) fn current_frame_mut(&mut self) -> Result<&mut CallFrame, RuntimeError> {
        self.frames.last_mut().ok_or(RuntimeError::InvalidFunction)
    }

    #[inline(always)]
    pub(crate) fn read_byte(&mut self) -> Result<u8, RuntimeError> {
        #[cfg(feature = "profile")]
        self.profile_read_byte();

        let frame = self
            .frames
            .last_mut()
            .ok_or(RuntimeError::InvalidFunction)?;

        if frame.ip >= frame.chunk.code.len() {
            return Err(RuntimeError::InvalidFunction);
        }

        let byte = frame.chunk.code[frame.ip];

        frame.ip += 1;

        Ok(byte)
    }
   
    #[inline(always)]
    pub(crate) fn read_short(&mut self) -> Result<u16, RuntimeError> {
        #[cfg(feature = "profile")]
        {
            self.profile_read_byte();
            self.profile_read_byte();
        }

        let frame = self
            .frames
            .last_mut()
            .ok_or(RuntimeError::InvalidFunction)?;

        let ip = frame.ip;

        if ip.checked_add(1).is_none() || ip + 1 >= frame.chunk.code.len() {
            return Err(RuntimeError::InvalidFunction);
        }

        let high = frame.chunk.code[ip] as u16;
        let low = frame.chunk.code[ip + 1] as u16;

        frame.ip = ip + 2;

        Ok((high << 8) | low)
    }

    #[inline]
    pub(crate) fn read_constant(&self, index: u8) -> Result<Value, RuntimeError> {
        self.current_frame()?
            .chunk
            .constants
            .get(index as usize)
            .cloned()
            .ok_or(RuntimeError::InvalidFunction)
    }

    #[inline(always)]
    pub(crate) fn read_constant_byte(&mut self) -> Result<Value, RuntimeError> {
        let index = self.read_byte()?;

        self.read_constant(index)
    }

    #[inline]
    pub(crate) fn current_position(&self) -> Result<(usize, usize), RuntimeError> {
        let frame = self.current_frame()?;

        Ok(frame.chunk.position_at(frame.ip))
    }
}

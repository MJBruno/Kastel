use super::VirtualMachine;

use crate::{
    error::runtime_error::RuntimeError,
    runtime::{object::Object, value::Value},
};

impl VirtualMachine {
    pub(crate) fn jump(&mut self) -> Result<(), RuntimeError> {
        let offset = self.read_short()? as usize;
        self.jump_forward(offset)
    }

    pub(crate) fn jump_if_false(&mut self) -> Result<(), RuntimeError> {
        let offset = self.read_short()? as usize;

        if !self.peek()?.is_truthy() {
            self.jump_forward(offset)?;
        }

        Ok(())
    }

    pub(crate) fn jump_if_false_pop(&mut self) -> Result<(), RuntimeError> {
        let offset = self.read_short()? as usize;

        let condition = self.pop()?;

        if !condition.is_truthy() {
            self.jump_forward(offset)?;
        }

        Ok(())
    }

    #[inline(always)]
    pub(crate) fn loop_back(&mut self) -> Result<(), RuntimeError> {
        let offset = self.read_short()? as usize;

        let frame = self
            .frames
            .last_mut()
            .ok_or(RuntimeError::InvalidFunction)?;

        let new_ip = frame
            .ip
            .checked_sub(offset)
            .ok_or(RuntimeError::InvalidFunction)?;

        if new_ip >= frame.chunk.code.len() {
            return Err(RuntimeError::InvalidFunction);
        }

        frame.ip = new_ip;

        Ok(())
    }

    fn jump_forward(&mut self, offset: usize) -> Result<(), RuntimeError> {
        let frame = self.current_frame()?;

        let new_ip = frame
            .ip
            .checked_add(offset)
            .ok_or(RuntimeError::InvalidFunction)?;

        if new_ip >= frame.chunk.code.len() {
            return Err(RuntimeError::InvalidFunction);
        }

        self.current_frame_mut()?.ip = new_ip;

        Ok(())
    }

    /// Runtime de l'opérateur `?`.
    ///
    /// Transforme le sommet en `(payload, true)` pour `Some/Ok`, ou en
    /// `(None/Err, false)` pour une propagation. Le compilateur transforme
    /// ensuite `false` en retour de la fonction courante.
    pub(crate) fn try_operator(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop()?;

        let outcome = match &value {
            Value::None => Some((Value::None, false)),

            Value::Object(handle) => {
                let object = handle.borrow();
                match &*object {
                    Object::Option(Some(inner)) => Some((inner.clone(), true)),
                    Object::Option(None) => Some((Value::None, false)),
                    Object::Result { ok: true, value } => Some((value.clone(), true)),
                    Object::Result { ok: false, .. } => Some((value.clone(), false)),
                    _ => None,
                }
            }

            _ => None,
        };

        let Some((result, success)) = outcome else {
            return Err(RuntimeError::TryOperandType {
                found: value.type_name().to_string(),
            });
        };

        self.push(result);
        self.push(Value::Boolean(success));
        Ok(())
    }

}

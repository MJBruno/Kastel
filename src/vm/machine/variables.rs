use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::VirtualMachine;

use crate::error::runtime_error::RuntimeError;
use crate::runtime::object::Object;
use crate::runtime::value::{NumericOp, Value};

impl VirtualMachine {
    fn current_global_env(&self) -> Rc<RefCell<HashMap<String, Value>>> {
        if let Some(frame) = self.frames.last() {
            let closure = super::bytecode::frame_closure(&frame.closure);

            if let Some(env) = closure.global_env.upgrade() {
                return env;
            }
        }

        Rc::clone(&self.globals)
    }

    pub(crate) fn define_global(&mut self, wide: bool) -> Result<(), RuntimeError> {
        let constant = self.read_constant_byte(wide)?;
        let name = constant.as_string_value().ok_or(RuntimeError::TypeError)?;

        let value = self.pop()?;
        self.current_global_env().borrow_mut().insert(name, value);

        Ok(())
    }

    /// `Overload <nom>` : la fonction au sommet de la pile rejoint
    /// l'ensemble de surcharges de la globale `nom`, qui doit déjà contenir
    /// une fonction de même nom (première déclaration).
    ///
    /// * globale = fonction seule -> elle devient un ensemble
    ///   `[ancienne, nouvelle]` ;
    /// * globale = ensemble -> la nouvelle y est ajoutée EN PLACE, donc une
    ///   valeur `let g = f;` déjà prise voit aussi la nouvelle surcharge.
    ///
    /// Deux surcharges de même arité restent refusées (le compilateur les
    /// détecte déjà ; ceci protège le cas d'un bytecode incohérent).
    pub(crate) fn op_overload(&mut self, wide: bool) -> Result<(), RuntimeError> {
        let constant = self.read_constant_byte(wide)?;
        let name = constant.as_string_value().ok_or(RuntimeError::TypeError)?;

        let function = self.pop()?;

        let new_arity = Self::function_arity(&function).ok_or(RuntimeError::TypeError)?;

        let globals = self.current_global_env();
        let existing = globals
            .borrow()
            .get(&name)
            .cloned()
            .ok_or(RuntimeError::TypeError)?;

        // Ensemble déjà constitué : ajout en place.
        if let Value::Object(handle) = &existing {
            let mut object = handle.borrow_mut();

            if let Object::Overloads { functions, .. } = &mut *object {
                if functions
                    .iter()
                    .any(|other| Self::function_arity(other) == Some(new_arity))
                {
                    return Err(RuntimeError::DuplicateMethod {
                        name,
                        arity: new_arity,
                    });
                }

                functions.push(function);

                return Ok(());
            }
        }

        // Première surcharge : la globale est encore une fonction seule.
        let existing_arity = Self::function_arity(&existing).ok_or(RuntimeError::TypeError)?;

        if existing_arity == new_arity {
            return Err(RuntimeError::DuplicateMethod {
                name,
                arity: new_arity,
            });
        }

        let set = Value::new_overloads(name.clone(), vec![existing, function]);

        globals.borrow_mut().insert(name, set);

        Ok(())
    }

    /// Nombre de paramètres d'une fermeture de fonction libre.
    pub(crate) fn function_arity(value: &Value) -> Option<usize> {
        match value {
            Value::Object(handle) => match &*handle.borrow() {
                Object::Closure(closure) => Some(closure.function.arity),
                _ => None,
            },

            _ => None,
        }
    }

    pub(crate) fn get_global(&mut self, wide: bool) -> Result<(), RuntimeError> {
        let constant = self.read_constant_byte(wide)?;
        let name = constant.as_string_value().ok_or(RuntimeError::TypeError)?;

        let globals = self.current_global_env();
        let value = globals
            .borrow()
            .get(&name)
            .cloned()
            .ok_or(RuntimeError::TypeError)?;

        self.push(value);

        Ok(())
    }

    pub(crate) fn set_global(&mut self, wide: bool) -> Result<(), RuntimeError> {
        let constant = self.read_constant_byte(wide)?;
        let name = constant.as_string_value().ok_or(RuntimeError::TypeError)?;

        let globals = self.current_global_env();

        if !globals.borrow().contains_key(&name) {
            return Err(RuntimeError::TypeError);
        }

        let value = self.peek()?.clone();
        globals.borrow_mut().insert(name, value);

        Ok(())
    }

    pub(crate) fn get_local(&mut self) -> Result<(), RuntimeError> {
        let slot = self.read_byte()? as usize;

        let (slot_start, local_count) = {
            let frame = self.frames.last().ok_or(RuntimeError::InvalidFunction)?;

            (frame.slot_start, frame.local_count)
        };

        if slot >= local_count {
            return Err(RuntimeError::InvalidFunction);
        }

        let index = slot_start
            .checked_add(1)
            .and_then(|index| index.checked_add(slot))
            .ok_or(RuntimeError::InvalidFunction)?;

        let value = self
            .stack
            .get(index)
            .cloned()
            .ok_or(RuntimeError::StackUnderflow)?;

        self.stack.push(value);

        Ok(())
    }

    pub(crate) fn set_local(&mut self) -> Result<(), RuntimeError> {
        let slot = self.read_byte()? as usize;

        let value = self
            .stack
            .last()
            .cloned()
            .ok_or(RuntimeError::StackUnderflow)?;

        let (slot_start, local_count) = {
            let frame = self.frames.last().ok_or(RuntimeError::InvalidFunction)?;

            (frame.slot_start, frame.local_count)
        };

        if slot >= local_count {
            return Err(RuntimeError::InvalidFunction);
        }

        let index = slot_start
            .checked_add(1)
            .and_then(|index| index.checked_add(slot))
            .ok_or(RuntimeError::InvalidFunction)?;

        let target = self
            .stack
            .get_mut(index)
            .ok_or(RuntimeError::StackUnderflow)?;

        *target = value;

        Ok(())
    }

    #[inline(always)]
    pub(crate) fn add_local_const(&mut self) -> Result<(), RuntimeError> {
        let slot = self.read_byte()? as usize;
        let constant_index = self.read_byte()? as usize;

        let (slot_start, constant) = {
            let frame = self.frames.last().ok_or(RuntimeError::InvalidFunction)?;

            if slot >= frame.local_count {
                return Err(RuntimeError::InvalidFunction);
            }

            let constant = frame
                .chunk
                .constants
                .get(constant_index)
                .ok_or(RuntimeError::InvalidFunction)?;

            (frame.slot_start, constant)
        };

        let local_index = slot_start
            .checked_add(1)
            .and_then(|index| index.checked_add(slot))
            .ok_or(RuntimeError::InvalidFunction)?;

        /*
         * Hot path :
         *
         *     Integer + Integer
         *
         * Aucun clone de la valeur locale et aucun appel au helper
         * générique.
         */
        if let (Some(Value::Integer(local)), Value::Integer(constant)) =
            (self.stack.get(local_index), constant)
        {
            let result = local
                .checked_add(*constant)
                .ok_or(RuntimeError::IntegerOverflow {
                    operation: "addition",
                })?;

            let target = self
                .stack
                .get_mut(local_index)
                .ok_or(RuntimeError::StackUnderflow)?;

            *target = Value::Integer(result);

            return Ok(());
        }

        /*
         * Fallback pour les autres combinaisons numériques.
         */
        let local = self
            .stack
            .get(local_index)
            .cloned()
            .ok_or(RuntimeError::StackUnderflow)?;

        let result = Value::binary_numeric_op(local, constant.clone(), NumericOp::Add)?;

        let target = self
            .stack
            .get_mut(local_index)
            .ok_or(RuntimeError::StackUnderflow)?;

        *target = result;

        Ok(())
    }
}

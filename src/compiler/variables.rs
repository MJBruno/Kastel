use std::rc::Rc;

use crate::bytecode::chunk::OpCode;
use crate::error::compile_error::CompileError;

use super::compiler::Compiler;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Global {
    pub constant: u8,
    pub mutable: bool,
}

/// Indique l'emplacement où une variable a été résolue par le compilateur.
/// Une variable peut appartenir à la portée locale, être globale ou être
/// capturée depuis une portée extérieure sous forme d'upvalue.
pub enum VariableLocation {
    Local(usize),
    Global,
    Upvalue(usize),
}

impl Compiler {
    // ============================================================
    //                      VARIABLES
    // ============================================================

    pub(crate) fn resolve_variable(&mut self, name: &str) -> Result<VariableLocation, CompileError> {
        // ============================================================
        // 1. LOCAL
        // ============================================================

        if let Some(slot) = self.context.borrow().locals.resolve_local(name)? {
            return Ok(VariableLocation::Local(slot as usize));
        }

        // ============================================================
        // 2. UPVALUE
        // ============================================================

        if let Some(slot) = self.resolve_upvalue(name)? {
            return Ok(VariableLocation::Upvalue(slot));
        }

        // ============================================================
        // 3. GLOBAL
        // ============================================================

        if self.globals.borrow().contains_key(name) {
            return Ok(VariableLocation::Global);
        }

        // ============================================================
        // 4. INEXISTANTE
        // ============================================================

        Err(CompileError::UndefinedVariable(name.to_string()))
    }

    pub(crate) fn compile_variable_get(&mut self, name: &str) -> Result<(), CompileError> {
        match self.resolve_variable(name)? {
            VariableLocation::Local(slot) => {
                self.emit_bytes(OpCode::GetLocal, slot as u8);
            }

            VariableLocation::Global => {
                let name_constant = self.identifier_constant(name)?;

                self.emit_bytes(OpCode::GetGlobal, name_constant);
            }

            VariableLocation::Upvalue(slot) => {
                self.emit_bytes(OpCode::GetUpvalue, slot as u8);
            }
        }

        Ok(())
    }

    pub(crate) fn compile_variable_set(&mut self, name: &str) -> Result<(), CompileError> {
        match self.resolve_variable(name)? {
            // ========================================================
            // LOCAL
            // ========================================================
            VariableLocation::Local(slot) => {
                if let Some(false) = self.context.borrow().locals.is_mutable(name)? {
                    return Err(CompileError::AssignmentToConstant(name.to_string()));
                }

                self.emit_bytes(OpCode::SetLocal, slot as u8);
            }

            // ========================================================
            // GLOBAL
            // ========================================================
            VariableLocation::Global => {
                let mutable = {
                    let globals = self.globals.borrow();

                    globals
                        .get(name)
                        .map(|global| global.mutable)
                        .unwrap_or(true)
                };

                if !mutable {
                    return Err(CompileError::AssignmentToConstant(name.to_string()));
                }

                let name_constant = self.identifier_constant(name)?;

                self.emit_bytes(OpCode::SetGlobal, name_constant);
            }

            // ========================================================
            // UPVALUE
            // ========================================================
            VariableLocation::Upvalue(slot) => {
                if !self.is_upvalue_mutable(name)? {
                    return Err(CompileError::AssignmentToConstant(name.to_string()));
                }

                self.emit_bytes(OpCode::SetUpvalue, slot as u8);
            }
        }

        Ok(())
    }

    pub(crate) fn is_upvalue_mutable(&self, name: &str) -> Result<bool, CompileError> {
        let mut context = {
            let current = self.context.borrow();

            match &current.enclosing {
                Some(parent) => Rc::clone(parent),
                None => return Ok(true),
            }
        };

        loop {
            let mutable = {
                let context_ref = context.borrow();

                context_ref.locals.is_mutable(name)?
            };

            if let Some(mutable) = mutable {
                return Ok(mutable);
            }

            let next = {
                let context_ref = context.borrow();

                match &context_ref.enclosing {
                    Some(parent) => Rc::clone(parent),
                    None => return Ok(true),
                }
            };

            context = next;
        }
    }
}
use crate::bytecode::chunk::OpCode;
use crate::error::compile_error::CompileError;
use crate::frontend::ast::Expression;

use super::compiler::Compiler;
use super::variables::Global;

impl Compiler {
    // ============================================================
    //                      VARIABLES DECLARATION
    // ============================================================

    pub(crate) fn compile_var(
        &mut self,
        name: &str,
        initializer: Option<&Expression>,
        mutable: bool,
    ) -> Result<(), CompileError> {
        if self.in_function || self.scope_depth > 0 {
            self.compile_local_var(name, initializer, mutable)
        } else {
            self.compile_global_var(name, initializer, mutable)
        }
    }

    pub(crate) fn compile_local_var(
        &mut self,
        name: &str,
        initializer: Option<&Expression>,
        mutable: bool,
    ) -> Result<(), CompileError> {
        let slot =
            self.context
                .borrow_mut()
                .locals
                .declare_local(name, self.scope_depth, mutable)?;

        match initializer {
            Some(expr) => {
                self.compile_expression(expr)?;
            }

            None => {
                self.emit_opcode(OpCode::Nil);
            }
        }

        self.context
            .borrow_mut()
            .locals
            .mark_initialized(self.scope_depth);

        debug_assert_eq!(self.context.borrow().locals.len() - 1, slot as usize);

        Ok(())
    }

    pub(crate) fn compile_global_var(
        &mut self,
        name: &str,
        initializer: Option<&Expression>,
        mutable: bool,
    ) -> Result<(), CompileError> {
        if self.globals.borrow().contains_key(name) {
            return Err(CompileError::VariableAlreadyDeclared(name.to_string()));
        }

        let name_constant = self.identifier_constant(name)?;

        match initializer {
            Some(expr) => {
                self.compile_expression(expr)?;
            }

            None => {
                self.emit_opcode(OpCode::Nil);
            }
        }

        self.emit_bytes(OpCode::DefineGlobal, name_constant);

        self.globals.borrow_mut().insert(
            name.to_string(),
            Global {
                constant: name_constant,
                mutable,
            },
        );

        Ok(())
    }

    // ============================================================
    //                      PARAMÈTRES
    // ============================================================

    pub(crate) fn add_parametre(&mut self, name: &str) -> Result<(), CompileError> {
        let slot = self
            .context
            .borrow_mut()
            .locals
            .declare_local(name, self.scope_depth, true)?;

        self.context
            .borrow_mut()
            .locals
            .mark_initialized(self.scope_depth);

        debug_assert_eq!(slot, self.function_arity);

        self.function_arity += 1;

        Ok(())
    }
}
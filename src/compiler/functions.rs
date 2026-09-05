use std::rc::Rc;

use crate::bytecode::chunk::OpCode;
use crate::error::compile_error::CompileError;
use crate::frontend::ast::Expression;
use crate::frontend::ast::Statement;
use crate::runtime::function::Function;
use crate::runtime::objet;
use crate::runtime::upvalue::Upvalue;
use crate::runtime::value::Value;

use super::compiler::Compiler;
use super::variables::Global;

impl Compiler {
    // ============================================================
    //                      CLOSURE
    // ============================================================

    pub(crate) fn emit_closure(&mut self, function_constant: u8, upvalues: &[Upvalue]) {
        self.emit_bytes(OpCode::Closure, function_constant);

        for upvalue in upvalues {
            self.emit_byte(if upvalue.is_local { 1 } else { 0 });

            self.emit_byte(upvalue.index);
        }
    }

    pub(crate) fn compile_function_statement(
        &mut self,
        name: &str,
        params: &[String],
        body: &[Statement],
    ) -> Result<(), CompileError> {
        // ========================================================
        // FONCTION GLOBALE
        // ========================================================

        if !self.in_function && self.scope_depth == 0 {
            if self.globals.borrow().contains_key(name) {
                return Err(CompileError::VariableAlreadyDeclared(name.to_string()));
            }

            let name_constant = self.identifier_constant(name)?;

            // Réserver le nom.
            self.globals.borrow_mut().insert(
                name.to_string(),
                Global {
                    constant: name_constant,
                    mutable: true,
                },
            );

            let function = self.compile_function(name, params, body)?;

            let function_constant =
                self.make_constant(objet::new_function(Rc::new(function.clone())))?;

            self.emit_closure(function_constant, &function.upvalues);

            self.emit_bytes(OpCode::DefineGlobal, name_constant);

            return Ok(());
        }

        // ========================================================
        // FONCTION LOCALE / NESTED
        // ========================================================

        let function = self.compile_function(name, params, body)?;

        let function_constant = self.make_constant(objet::new_function(Rc::new(function.clone())))?;

        self.emit_closure(function_constant, &function.upvalues);

        let slot = self
            .context
            .borrow_mut()
            .locals
            .declare_local(name, self.scope_depth, true)?;

        self.context
            .borrow_mut()
            .locals
            .mark_initialized(self.scope_depth);

        debug_assert_eq!(self.context.borrow().locals.len() - 1, slot as usize);

        Ok(())
    }

    // ========================================================
    //                      COMPILE_FONCTION
    // ========================================================

    pub(crate) fn compile_function(
        &mut self,
        name: &str,
        params: &[String],
        body: &[Statement],
    ) -> Result<Function, CompileError> {
        let enclosing = Rc::clone(&self.context);

        let mut compiler =
            Compiler::new_function(name.to_string(), Rc::clone(&self.globals), enclosing);

        for param in params {
            compiler.add_parametre(param)?;
        }

        for statement in body {
            compiler.compile_statement(statement)?;
        }

        compiler.emit_opcode(OpCode::Nil);

        compiler.emit_opcode(OpCode::Return);

        let upvalues = compiler.context.borrow().upvalues.clone();

        Ok(Function {
            name: name.to_string(),
            arity: compiler.function_arity as usize,
            chunk: compiler.chunk,
            upvalue_count: upvalues.len(),
            upvalues,
        })
    }

    // ============================================================
    //                      RETURN
    // ============================================================

    pub(crate) fn compile_return(&mut self, value: Option<&Expression>) -> Result<(), CompileError> {
        if !self.in_function {
            return Err(CompileError::ReturnOutsidFunction);
        }

        match value {
            Some(expression) => {
                self.compile_expression(expression)?;
            }

            None => {
                self.emit_opcode(OpCode::Nil);
            }
        }

        self.emit_opcode(OpCode::Return);

        Ok(())
    }
}
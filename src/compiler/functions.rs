use std::rc::Rc;

use crate::bytecode::chunk::OpCode;
use crate::error::compile_error::CompileError;
use crate::frontend::ast::Expression;
use crate::frontend::ast::Statement;
use crate::runtime::function::Function;
use crate::runtime::upvalue::Upvalue;
use crate::runtime::value::Value;

use super::compiler::Compiler;
use super::variables::Global;

impl Compiler {
    // ============================================================
    // CLOSURE
    // ============================================================

    pub(crate) fn emit_closure(&mut self, function_constant: u16, upvalues: &[Upvalue]) {
        self.emit_constant_op(OpCode::Closure, function_constant);

        for upvalue in upvalues {
            self.emit_byte(if upvalue.is_local { 1 } else { 0 });
            self.emit_byte(upvalue.index);
        }
    }

    // ============================================================
    // FUNCTION STATEMENT
    // ============================================================

    pub(crate) fn compile_function_statement(
        &mut self,
        name: &str,
        params: &[String],
        body: &[Statement],
    ) -> Result<(), CompileError> {
        // ========================================================
        // GLOBAL FUNCTION
        // ========================================================

        if !self.in_function && self.scope_depth == 0 {
            let name_constant = if self.predeclared_functions.contains(name) {
                self.globals
                    .borrow()
                    .get(name)
                    .map(|global| global.constant)
                    .ok_or_else(|| CompileError::VariableAlreadyDeclared(name.to_string()))?
            } else {
                // Chaque compilation du REPL possède son propre Chunk.
                // L'indice de constante enregistré dans un Global provenant
                // d'une compilation précédente appartient donc à un ancien
                // Chunk et ne peut pas être réutilisé.
                //
                // Une fonction globale peut être redéclarée dans le REPL
                // (redéfinition) ou dans un contexte sans pré-passage
                // (surcharge). Les collisions avec une variable, une classe,
                // etc. restent interdites.

                let existing = self.globals.borrow().get(name).cloned();

                if let Some(global) = &existing
                    && !global.native
                    && !global.is_function
                {
                    return Err(CompileError::VariableAlreadyDeclared(name.to_string()));
                }

                // Toujours créer une constante dans le Chunk courant.
                let constant = self.identifier_constant(name)?;

                self.globals.borrow_mut().insert(
                    name.to_string(),
                    Global {
                        constant,
                        mutable: true,
                        native: false,
                        is_function: true,
                    },
                );

                constant
            };

            let function = self.compile_function(name, params, body)?;

            let function_constant =
                self.make_constant(Value::new_function(Rc::new(function.clone())))?;

            self.emit_closure(function_constant, &function.upvalues);

            // `Overload` couvre :
            //
            // - la première déclaration ;
            // - une surcharge ;
            // - une redéfinition dans le REPL.
            //
            // Dans le REPL, chaque ligne possède son propre Chunk. Le
            // `name_constant` utilisé ici appartient donc toujours au Chunk
            // actuellement compilé.
            self.emit_constant_op(OpCode::Overload, name_constant);

            return Ok(());
        }

        // ========================================================
        // LOCAL / NESTED FUNCTION
        // ========================================================

        // Surcharge locale : une fonction du même nom est déjà déclarée
        // dans CETTE portée (même arité = erreur).
        let overload_target = self
            .context
            .borrow()
            .locals
            .local_function_in_scope(name, self.scope_depth);

        if let Some((_, arities)) = &overload_target
            && arities.contains(&params.len())
        {
            return Err(CompileError::DuplicateFunction {
                name: name.to_string(),
                arity: params.len(),
            });
        }

        let function = self.compile_function(name, params, body)?;

        let function_constant =
            self.make_constant(Value::new_function(Rc::new(function.clone())))?;

        self.emit_closure(function_constant, &function.upvalues);

        if let Some((existing_slot, _)) = overload_target {
            // La fermeture est au sommet de la pile : `OverloadLocal`
            // la retire et l'ajoute à l'ensemble de la locale
            // `existing_slot`.
            self.emit_bytes(OpCode::OverloadLocal, existing_slot);

            self.context
                .borrow_mut()
                .locals
                .add_function_arity(existing_slot, params.len());

            return Ok(());
        }

        let slot = self
            .context
            .borrow_mut()
            .locals
            .declare_local(name, self.scope_depth, true)?;

        self.context
            .borrow_mut()
            .locals
            .mark_initialized(self.scope_depth);

        self.context
            .borrow_mut()
            .locals
            .add_function_arity(slot, params.len());

        debug_assert_eq!(self.context.borrow().locals.len() - 1, slot as usize);

        Ok(())
    }

    // ============================================================
    // COMPILE FUNCTION
    // ============================================================

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

        compiler.emit_opcode(OpCode::None);
        compiler.emit_opcode(OpCode::Return);

        let (upvalue_count, upvalues, local_count) = {
            let context = compiler.context.borrow();

            (
                context.upvalues.len(),
                context.upvalues.clone(),
                u16::try_from(context.locals.max_slots())
                    .map_err(|_| CompileError::TooManyLocals)?,
            )
        };

        Ok(Function {
            name: name.to_string(),
            arity: compiler.function_arity as usize,
            chunk: Rc::new(compiler.chunk),
            local_count,
            upvalue_count,
            upvalues,
        })
    }

    // ============================================================
    // RETURN
    // ============================================================

    pub(crate) fn compile_return(
        &mut self,
        value: Option<&Expression>,
    ) -> Result<(), CompileError> {
        if !self.in_function {
            return Err(CompileError::ReturnOutsidFunction);
        }

        match value {
            Some(expression) => self.compile_expression(expression)?,

            None => self.emit_opcode(OpCode::None),
        }

        self.compile_active_finally()?;

        self.emit_opcode(OpCode::Return);

        Ok(())
    }

    // ============================================================
    // METHOD
    // ============================================================

    pub(crate) fn compile_method(
        &mut self,
        name: &str,
        params: &[String],
        body: &[Statement],
    ) -> Result<Function, CompileError> {
        let mut method_params = Vec::with_capacity(params.len() + 1);

        method_params.push("this".to_string());
        method_params.extend(params.iter().cloned());

        self.compile_function(name, &method_params, body)
    }

    // ============================================================
    // STATIC METHOD
    // ============================================================

    /// Comme `compile_method`, mais SANS `this` implicite : une méthode
    /// `static` n'est pas appelée sur une instance (`NomClasse.methode(...)`),
    /// donc son corps ne reçoit aucun receveur.
    pub(crate) fn compile_static_method(
        &mut self,
        name: &str,
        params: &[String],
        body: &[Statement],
    ) -> Result<Function, CompileError> {
        self.compile_function(name, params, body)
    }
}

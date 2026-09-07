use crate::bytecode::chunk::OpCode;
use crate::error::compile_error::CompileError;
use crate::frontend::ast::{Expression, Statement};

use super::compiler::Compiler;

impl Compiler {
    // ============================================================
    //                      COMPILE_IF
    // ============================================================

    pub(crate) fn compile_if(
        &mut self,
        condition: &Expression,
        then_branch: &[Statement],
        else_branch: Option<&Vec<Statement>>,
    ) -> Result<(), CompileError> {
        // Évalue la condition.
        self.compile_expression(condition)?;

        // Si la condition est false, aller au bloc else.
        let else_jump = self.emit_jump(OpCode::JumpIfFalse);

        // --------------------------------------------------------
        // THEN
        // --------------------------------------------------------

        for statement in then_branch {
            self.compile_statement(statement)?;
        }

        // La condition est encore sur la pile lorsque le THEN
        // est exécuté : on la retire.
        self.emit_opcode(OpCode::Pop);

        // S'il existe un ELSE, sauter par-dessus après le THEN.
        let end_jump = if else_branch.is_some() {
            Some(self.emit_jump(OpCode::Jump))
        } else {
            None
        };

        // --------------------------------------------------------
        // ELSE
        // --------------------------------------------------------

        self.patch_jump(else_jump);

        // La condition false est toujours sur la pile.
        self.emit_opcode(OpCode::Pop);

        if let Some(else_branch) = else_branch {
            for statement in else_branch {
                self.compile_statement(statement)?;
            }
        }

        // --------------------------------------------------------
        // END
        // --------------------------------------------------------

        if let Some(end_jump) = end_jump {
            self.patch_jump(end_jump);
        }

        Ok(())
    }
}
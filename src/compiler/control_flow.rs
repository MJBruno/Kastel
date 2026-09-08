use crate::bytecode::chunk::OpCode;
use crate::error::compile_error::CompileError;
use crate::frontend::ast::{Expression, Statement};

use super::compiler::Compiler;

impl Compiler {
    pub(crate) fn compile_if(
        &mut self,
        condition: &Expression,
        then_branch: &[Statement],
        else_branch: Option<&Vec<Statement>>,
    ) -> Result<(), CompileError> {
        // --------------------------------------------------------
        // CONDITION
        // --------------------------------------------------------
        self.compile_expression(condition)?;

        // JumpIfFalse conserve la condition sur la pile.
        let else_jump = self.emit_jump(OpCode::JumpIfFalse);

        // --------------------------------------------------------
        // THEN
        // --------------------------------------------------------
        // La condition est vraie : on peut la supprimer maintenant.
        self.emit_opcode(OpCode::Pop);

        for statement in then_branch {
            self.compile_statement(statement)?;
        }

        // Toujours sauter par-dessus le ELSE après le THEN.
        let end_jump = self.emit_jump(OpCode::Jump);

        // --------------------------------------------------------
        // ELSE
        // --------------------------------------------------------
        self.patch_jump(else_jump)?;

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
        self.patch_jump(end_jump)?;

        Ok(())
    }
}
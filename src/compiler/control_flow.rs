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
        self.compile_expression(condition)?;

        let then_jump = self.emit_jump(OpCode::JumpIfFalse);

        for statement in then_branch {
            self.compile_statement(statement)?;
        }

        if let Some(else_branch) = else_branch {
            let else_jump = self.emit_jump(OpCode::Jump);

            self.patch_jump(then_jump);

            self.emit_opcode(OpCode::Pop);

            for statement in else_branch {
                self.compile_statement(statement)?;
            }

            self.patch_jump(else_jump);
        } else {
            self.patch_jump(then_jump);

            self.emit_opcode(OpCode::Pop);
        }

        Ok(())
    }
}
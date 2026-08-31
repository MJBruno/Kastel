use crate::bytecode::chunk::OpCode;
use crate::error::compile_error::CompileError;
use crate::frontend::ast::{Expression, Statement};
use crate::runtime::value::Value;

use super::compiler::Compiler;

/// État de compilation d'une boucle actuellement active.
/// Cet état permet de résoudre correctement `break` et `continue` après
/// génération du bytecode.
pub struct LoopContext {
    /// Offset de bytecode vers lequel `continue` doit revenir.
    pub continue_target: usize,
    /// Liste des sauts `break` qui devront être corrigés à la fin de la boucle.
    pub break_jumps: Vec<usize>,
    /// Profondeur de portée à laquelle la boucle a été créée.
    pub scope_depth: usize,
}

impl Compiler {
    // ============================================================
    //                      WHILE
    // ============================================================

    pub(crate) fn compile_while(
        &mut self,
        condition: &Expression,
        body: &[Statement],
    ) -> Result<(), CompileError> {
        let loop_start = self.chunk.code.len();

        self.compile_expression(condition)?;

        let exit_jump = self.emit_jump(OpCode::JumpIfFalse);

        self.emit_opcode(OpCode::Pop);

        self.loops.push(LoopContext {
            continue_target: loop_start,
            break_jumps: Vec::new(),
            scope_depth: self.scope_depth,
        });

        self.begin_scope();

        for statement in body {
            self.compile_statement(statement)?;
        }

        self.end_scope();

        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);

        self.emit_opcode(OpCode::Pop);

        let loop_context = self.loops.pop().expect("loop stack underflow");

        for break_jump in loop_context.break_jumps {
            self.patch_jump(break_jump);
        }

        Ok(())
    }

    // ============================================================
    //                      FOR..IN
    // ============================================================

    pub(crate) fn compile_for_in(
        &mut self,
        variable: &str,
        iterable: &Expression,
        body: &[Statement],
    ) -> Result<(), CompileError> {
        self.begin_scope();

        // ============================================================
        // __for_iterable
        // ============================================================

        let iterable_slot = self.context.borrow_mut().locals.declare_local(
            "__for_iterable",
            self.scope_depth,
            false,
        )?;

        self.compile_expression(iterable)?;

        self.context
            .borrow_mut()
            .locals
            .mark_initialized(self.scope_depth);

        // ============================================================
        // __for_index = 0
        // ============================================================

        let index_slot = self.context.borrow_mut().locals.declare_local(
            "__for_index",
            self.scope_depth,
            true,
        )?;

        let zero = self.make_constant(Value::Number(0.0))?;
        self.emit_bytes(OpCode::Constant, zero);

        self.context
            .borrow_mut()
            .locals
            .mark_initialized(self.scope_depth);

        // ============================================================
        // Variable implicite du for
        //
        // for i in range(5) { ... }
        //
        // i devient automatiquement une locale.
        // ============================================================

        let variable_slot =
            self.context
                .borrow_mut()
                .locals
                .declare_local(variable, self.scope_depth, true)?;

        self.emit_opcode(OpCode::Nil);

        self.context
            .borrow_mut()
            .locals
            .mark_initialized(self.scope_depth);

        // ============================================================
        // Premier passage -> condition
        // ============================================================

        let initial_jump = self.emit_jump(OpCode::Jump);

        // ============================================================
        // CONTINUE -> INCREMENT
        // ============================================================

        let continue_target = self.chunk.code.len();

        self.emit_bytes(OpCode::GetLocal, index_slot);

        let one = self.make_constant(Value::Number(1.0))?;
        self.emit_bytes(OpCode::Constant, one);

        self.emit_opcode(OpCode::Add);

        self.emit_bytes(OpCode::SetLocal, index_slot);
        self.emit_opcode(OpCode::Pop);

        // ============================================================
        // INCREMENT -> CONDITION
        // ============================================================

        let condition_jump = self.emit_jump(OpCode::Jump);

        // ============================================================
        // CONDITION
        // ============================================================

        self.patch_jump(initial_jump);
        self.patch_jump(condition_jump);

        self.emit_bytes(OpCode::GetLocal, index_slot);
        self.emit_bytes(OpCode::GetLocal, iterable_slot);
        self.emit_opcode(OpCode::ArrayLength);
        self.emit_opcode(OpCode::Less);

        let exit_jump = self.emit_jump(OpCode::JumpIfFalse);

        self.emit_opcode(OpCode::Pop);

        // ============================================================
        // i = iterable[index]
        // ============================================================

        self.emit_bytes(OpCode::GetLocal, iterable_slot);
        self.emit_bytes(OpCode::GetLocal, index_slot);
        self.emit_opcode(OpCode::GetIndex);

        self.emit_bytes(OpCode::SetLocal, variable_slot);
        self.emit_opcode(OpCode::Pop);

        // ============================================================
        // LOOP CONTEXT
        // ============================================================

        self.loops.push(LoopContext {
            continue_target,
            break_jumps: Vec::new(),
            scope_depth: self.scope_depth,
        });


        self.begin_scope();

        for statement in body {
            self.compile_statement(statement)?;
        }

        self.end_scope();

        // ============================================================
        // BODY -> INCREMENT
        // ============================================================

        self.emit_loop(continue_target);

        // ============================================================
        // SORTIE
        // ============================================================

        self.patch_jump(exit_jump);

        self.emit_opcode(OpCode::Pop);

        // ============================================================
        // BREAK
        // ============================================================

        let loop_context = self.loops.pop().expect("loop stack underflow");

        for break_jump in loop_context.break_jumps {
            self.patch_jump(break_jump);
        }

        // ============================================================
        // FIN DU SCOPE
        // ============================================================

        self.end_scope();

        Ok(())
    }

    // ============================================================
    //                      BREAK / CONTINUE
    // ============================================================

    pub(crate) fn compile_break(&mut self) -> Result<(), CompileError> {
        let loop_depth = match self.loops.last() {
            Some(loop_context) => loop_context.scope_depth,

            None => return Err(CompileError::BreakOutsideLoop),
        };

        self.emit_scope_cleanup(loop_depth);

        let jump = self.emit_jump(OpCode::Jump);

        self.loops.last_mut().unwrap().break_jumps.push(jump);

        Ok(())
    }

    pub(crate) fn compile_continue(&mut self) -> Result<(), CompileError> {
        let (continue_target, loop_depth) = match self.loops.last() {
            Some(loop_context) => (loop_context.continue_target, loop_context.scope_depth),

            None => return Err(CompileError::ContinueOutsideLoop),
        };

        self.emit_scope_cleanup(loop_depth);

        self.emit_loop(continue_target);

        Ok(())
    }
}

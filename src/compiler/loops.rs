use crate::bytecode::chunk::OpCode;
use crate::error::compile_error::CompileError;
use crate::frontend::ast::{Expression, Statement};

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
    //
    // Désucrage générique via le protocole d'itération, PLUS AUCUNE
    // dépendance codée en dur sur les tableaux (ArrayLength/GetIndex) :
    //
    //   { iterable } -> GetIterator -> @for_iterator
    //
    //   while (@for_iterator.has_next()) {
    //       let variable = @for_iterator.next();
    //       body
    //   }
    //
    // Fonctionne donc identiquement pour un tableau, pour un Range
    // paresseux issu de range() (aucune allocation de tableau, même pour
    // range(1_000_000_000)), et pour tout futur type itérable — sans
    // toucher à cette fonction.
    //
    // Bonus architectural : plus besoin du hack "émettre l'incrément avant
    // le corps mais sauter par-dessus au premier passage" qu'exigeait
    // l'ancien désucrage façon for-C. IteratorNext EST l'avancement ; il
    // n'a lieu qu'une fois par itération, au bon endroit naturellement.
    // `continue` peut donc sauter directement au test has_next().

    pub(crate) fn compile_for_in(
        &mut self,
        variable: &str,
        iterable: &Expression,
        body: &[Statement],
    ) -> Result<(), CompileError> {
        self.begin_scope();

        // ------------------------------------------------------------
        // @for_iterator = GetIterator(iterable)
        // ------------------------------------------------------------

        self.compile_expression(iterable)?;

        self.emit_opcode(OpCode::GetIterator);

        let iterator_slot = self.context.borrow_mut().locals.declare_local(
            "@for_iterator",
            self.scope_depth,
            false,
        )?;

        self.context
            .borrow_mut()
            .locals
            .mark_initialized(self.scope_depth);

        // ------------------------------------------------------------
        // CONDITION : @for_iterator.has_next()
        // ------------------------------------------------------------

        let loop_start = self.chunk.code.len();

        self.emit_bytes(OpCode::GetLocal, iterator_slot);
        self.emit_opcode(OpCode::IteratorHasNext);

        let exit_jump = self.emit_jump(OpCode::JumpIfFalse);

        self.emit_opcode(OpCode::Pop); // dépile le booléen "true"

        self.loops.push(LoopContext {
            continue_target: loop_start,
            break_jumps: Vec::new(),
            scope_depth: self.scope_depth,
        });

        self.begin_scope();

        // ------------------------------------------------------------
        // variable = @for_iterator.next()
        // ------------------------------------------------------------

        self.emit_bytes(OpCode::GetLocal, iterator_slot);
        self.emit_opcode(OpCode::IteratorNext);

        self.context
            .borrow_mut()
            .locals
            .declare_local(variable, self.scope_depth, true)?;

        self.context
            .borrow_mut()
            .locals
            .mark_initialized(self.scope_depth);

        for statement in body {
            self.compile_statement(statement)?;
        }

        self.end_scope();

        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);

        self.emit_opcode(OpCode::Pop); // dépile le booléen "false"

        let loop_context = self.loops.pop().expect("loop stack underflow");

        for break_jump in loop_context.break_jumps {
            self.patch_jump(break_jump);
        }

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
use crate::bytecode::chunk::OpCode;
use crate::error::compile_error::CompileError;
use crate::frontend::ast::{BinaryOp, Expression, Literal, Statement};
use crate::runtime::value::Value;

use super::compiler::Compiler;
use super::variables::VariableLocation;

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

        /*
         * Fast path :
         *
         *     while i < 1000000 {
         *         ...
         *     }
         *
         * devient :
         *
         *     LessLocalConst <slot> <constant>
         *     JumpIfFalsePop <offset>
         *
         * au lieu de :
         *
         *     GetLocal
         *     Constant
         *     Less
         *     JumpIfFalse
         *     Pop
         *
         * LessLocalConst laisse le booléen sur la pile.
         * JumpIfFalsePop le consomme directement.
         */
        let optimized_condition = match condition {
            Expression::Binary {
                left,
                operator: BinaryOp::Less,
                right,
            } => {
                let local_name = match left.as_ref() {
                    Expression::Variable(name) => name,
                    _ => "",
                };

                if local_name.is_empty() {
                    false
                } else {
                    let constant_value = match right.as_ref() {
                        Expression::Literal(Literal::Integer(value)) => {
                            Some(Value::Integer(*value))
                        }

                        Expression::Literal(Literal::Float(value)) => {
                            Some(Value::Float(*value))
                        }

                        _ => None,
                    };

                    match constant_value {
                        Some(value) => match self.resolve_variable(local_name)? {
                            VariableLocation::Local(slot) => {
                                let constant = self.make_constant(value)?;

                                self.emit_bytes(
                                    OpCode::LessLocalConst,
                                    slot as u8,
                                );

                                self.emit_byte(constant);

                                true
                            }

                            _ => false,
                        },

                        None => false,
                    }
                }
            }

            _ => false,
        };

        if !optimized_condition {
            self.compile_expression(condition)?;
        }

        /*
         * Pour la condition optimisée comme pour la condition normale,
         * JumpIfFalsePop consomme le résultat booléen.
         *
         * Cela permet d'éviter :
         *
         *     JumpIfFalse
         *     Pop
         */
        let exit_jump = self.emit_jump(OpCode::JumpIfFalsePop);

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

        self.emit_loop(loop_start)?;

        self.patch_jump(exit_jump)?;

        let loop_context = self.loops.pop().ok_or_else(|| {
            CompileError::InternalCompilerError(
                "pile des boucles désynchronisée".to_string(),
            )
        })?;

        for break_jump in loop_context.break_jumps {
            self.patch_jump(break_jump)?;
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

        // ------------------------------------------------------------
        // @for_iterator = GetIterator(iterable)
        // ------------------------------------------------------------

        self.compile_expression(iterable)?;

        self.emit_opcode(OpCode::GetIterator);

        let iterator_slot = self
            .context
            .borrow_mut()
            .locals
            .declare_local(
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

        self.emit_bytes(
            OpCode::GetLocal,
            iterator_slot,
        );

        self.emit_opcode(OpCode::IteratorHasNext);

        let exit_jump =
            self.emit_jump(OpCode::JumpIfFalse);

        self.emit_opcode(OpCode::Pop);

        self.loops.push(LoopContext {
            continue_target: loop_start,
            break_jumps: Vec::new(),
            scope_depth: self.scope_depth,
        });

        self.begin_scope();

        // ------------------------------------------------------------
        // variable = @for_iterator.next()
        // ------------------------------------------------------------

        self.emit_bytes(
            OpCode::GetLocal,
            iterator_slot,
        );

        self.emit_opcode(OpCode::IteratorNext);

        self.context
            .borrow_mut()
            .locals
            .declare_local(
                variable,
                self.scope_depth,
                true,
            )?;

        self.context
            .borrow_mut()
            .locals
            .mark_initialized(self.scope_depth);

        for statement in body {
            self.compile_statement(statement)?;
        }

        self.end_scope();

        self.emit_loop(loop_start)?;

        self.patch_jump(exit_jump)?;

        self.emit_opcode(OpCode::Pop);

        let loop_context = self.loops.pop().ok_or_else(|| {
            CompileError::InternalCompilerError(
                "pile des boucles désynchronisée".to_string(),
            )
        })?;

        for break_jump in loop_context.break_jumps {
            self.patch_jump(break_jump)?;
        }

        self.end_scope();

        Ok(())
    }

    // ============================================================
    //                      BREAK / CONTINUE
    // ============================================================

    pub(crate) fn compile_break(
        &mut self,
    ) -> Result<(), CompileError> {
        let loop_depth = match self.loops.last() {
            Some(loop_context) => loop_context.scope_depth,

            None => {
                return Err(
                    CompileError::BreakOutsideLoop
                );
            }
        };

        self.emit_scope_cleanup(loop_depth);

        let jump = self.emit_jump(OpCode::Jump);

        self.loops
            .last_mut()
            .ok_or_else(|| {
                CompileError::InternalCompilerError(
                    "pile des boucles désynchronisée"
                        .to_string(),
                )
            })?
            .break_jumps
            .push(jump);

        Ok(())
    }

    pub(crate) fn compile_continue(
        &mut self,
    ) -> Result<(), CompileError> {
        let (continue_target, loop_depth) =
            match self.loops.last() {
                Some(loop_context) => (
                    loop_context.continue_target,
                    loop_context.scope_depth,
                ),

                None => {
                    return Err(
                        CompileError::ContinueOutsideLoop
                    );
                }
            };

        self.emit_scope_cleanup(loop_depth);

        self.emit_loop(continue_target)?;

        Ok(())
    }
}
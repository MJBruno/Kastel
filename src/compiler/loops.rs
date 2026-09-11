use crate::bytecode::chunk::OpCode;
use crate::error::compile_error::CompileError;
use crate::frontend::ast::{AssignmentTarget, BinaryOp, Expression, Literal, Statement};
use crate::runtime::value::Value;

use super::compiler::Compiler;
use super::variables::VariableLocation;

/// État de compilation d'une boucle actuellement active.
#[derive(Debug)]
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
    //                              WHILE
    // ============================================================

    pub(crate) fn compile_while(
        &mut self,
        condition: &Expression,
        body: &[Statement],
    ) -> Result<(), CompileError> {
        let loop_start = self.chunk.code.len();

        // --------------------------------------------------------
        // SUPER-INSTRUCTION COMPLÈTEMENT FUSIONNÉE
        //
        //     while i < 1000000 {
        //         i = i + 1;
        //     }
        //
        // devient :
        //
        //     LoopLessAddLocalConst
        //         slot
        //         limit_constant
        //         increment_constant
        //
        // Elle effectue directement :
        //
        //     if local >= limit {
        //         sortir de la boucle
        //     }
        //
        //     local += increment
        //     revenir au début de l'instruction
        //
        // On limite volontairement cette optimisation à un body
        // contenant exactement une affectation locale simple.
        // --------------------------------------------------------

        let fused_loop = if body.len() == 1 {
            let statement = match &body[0] {
                Statement::Positioned { statement, .. } => statement.as_ref(),
                statement => statement,
            };

            match statement {
                Statement::Assignment { target, value } => {
                    let variable_name = match target {
                        AssignmentTarget::Variable(name) => Some(name.as_str()),
                        _ => None,
                    };

                    let assignment = match value {
                        Expression::Binary {
                            left,
                            operator: BinaryOp::Add,
                            right,
                        } => {
                            let same_local = match (variable_name, left.as_ref()) {
                                (Some(name), Expression::Variable(left_name)) => name == left_name,
                                _ => false,
                            };

                            let increment = match right.as_ref() {
                                Expression::Literal(Literal::Integer(value)) => {
                                    Some(Value::Integer(*value))
                                }

                                Expression::Literal(Literal::Float(value)) => {
                                    Some(Value::Float(*value))
                                }

                                _ => None,
                            };

                            if same_local { increment } else { None }
                        }

                        _ => None,
                    };

                    match (variable_name, assignment) {
                        (Some(name), Some(increment)) => match condition {
                            Expression::Binary {
                                left: condition_left,
                                operator: BinaryOp::Less,
                                right: condition_right,
                            } => {
                                let condition_name = match condition_left.as_ref() {
                                    Expression::Variable(name) => Some(name.as_str()),
                                    _ => None,
                                };

                                let limit = match condition_right.as_ref() {
                                    Expression::Literal(Literal::Integer(value)) => {
                                        Some(Value::Integer(*value))
                                    }

                                    Expression::Literal(Literal::Float(value)) => {
                                        Some(Value::Float(*value))
                                    }

                                    _ => None,
                                };

                                if condition_name != Some(name) {
                                    false
                                } else {
                                    match (limit, self.resolve_variable(name)?) {
                                        (Some(limit), VariableLocation::Local(slot)) => {
                                            let limit_constant = self.make_constant(limit)?;

                                            let increment_constant =
                                                self.make_constant(increment)?;

                                            self.emit_opcode(OpCode::LoopLessAddLocalConst);

                                            self.emit_byte(slot as u8);
                                            self.emit_byte(limit_constant);
                                            self.emit_byte(increment_constant);

                                            true
                                        }

                                        _ => false,
                                    }
                                }
                            }

                            _ => false,
                        },

                        _ => false,
                    }
                }

                _ => false,
            }
        } else {
            false
        };
        if fused_loop {
            return Ok(());
        }

        // --------------------------------------------------------
        // CONDITION OPTIMISÉE :
        //
        //     while i < constant {
        //
        // devient :
        //
        //     LessLocalConstJump
        //         slot
        //         constant
        //         offset
        //
        // au lieu de :
        //
        //     LessLocalConst
        //     JumpIfFalsePop
        // --------------------------------------------------------

        let exit_jump = match condition {
            Expression::Binary {
                left,
                operator: BinaryOp::Less,
                right,
            } => {
                let local_name = match left.as_ref() {
                    Expression::Variable(name) => Some(name.as_str()),
                    _ => None,
                };

                let constant_value = match right.as_ref() {
                    Expression::Literal(Literal::Integer(value)) => Some(Value::Integer(*value)),

                    Expression::Literal(Literal::Float(value)) => Some(Value::Float(*value)),

                    _ => None,
                };

                match (local_name, constant_value) {
                    (Some(local_name), Some(value)) => {
                        match self.resolve_variable(local_name)? {
                            VariableLocation::Local(slot) => {
                                let constant = self.make_constant(value)?;

                                self.emit_opcode(OpCode::LessLocalConstJump);
                                self.emit_byte(slot as u8);
                                self.emit_byte(constant);

                                // Réserve les deux octets du saut.
                                self.emit_u16(u16::MAX);

                                self.chunk.code.len() - 2
                            }

                            _ => self.compile_condition_and_jump(condition)?,
                        }
                    }

                    _ => self.compile_condition_and_jump(condition)?,
                }
            }

            _ => self.compile_condition_and_jump(condition)?,
        };

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
            CompileError::InternalCompilerError("pile des boucles désynchronisée".to_string())
        })?;

        for break_jump in loop_context.break_jumps {
            self.patch_jump(break_jump)?;
        }

        Ok(())
    }

    // ============================================================
    //                       CONDITION + JUMP
    // ============================================================

    fn compile_condition_and_jump(
        &mut self,
        condition: &Expression,
    ) -> Result<usize, CompileError> {
        self.compile_expression(condition)?;

        Ok(self.emit_jump(OpCode::JumpIfFalsePop))
    }

    // ============================================================
    //                           FOR..IN
    // ============================================================

    pub(crate) fn compile_for_in(
        &mut self,
        variable: &str,
        iterable: &Expression,
        body: &[Statement],
    ) -> Result<(), CompileError> {
        self.begin_scope();

        // --------------------------------------------------------
        // @for_iterator = GetIterator(iterable)
        // --------------------------------------------------------

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

        // --------------------------------------------------------
        // CONDITION
        // --------------------------------------------------------

        let loop_start = self.chunk.code.len();

        self.emit_bytes(OpCode::GetLocal, iterator_slot);

        self.emit_opcode(OpCode::IteratorHasNext);

        let exit_jump = self.emit_jump(OpCode::JumpIfFalse);

        self.emit_opcode(OpCode::Pop);

        self.loops.push(LoopContext {
            continue_target: loop_start,
            break_jumps: Vec::new(),
            scope_depth: self.scope_depth,
        });

        self.begin_scope();

        // --------------------------------------------------------
        // variable = @for_iterator.next()
        // --------------------------------------------------------

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

        self.emit_loop(loop_start)?;

        self.patch_jump(exit_jump)?;

        self.emit_opcode(OpCode::Pop);

        let loop_context = self.loops.pop().ok_or_else(|| {
            CompileError::InternalCompilerError("pile des boucles désynchronisée".to_string())
        })?;

        for break_jump in loop_context.break_jumps {
            self.patch_jump(break_jump)?;
        }

        self.end_scope();

        Ok(())
    }

    // ============================================================
    //                         BREAK / CONTINUE
    // ============================================================

    pub(crate) fn compile_break(&mut self) -> Result<(), CompileError> {
        let loop_depth = match self.loops.last() {
            Some(loop_context) => loop_context.scope_depth,

            None => {
                return Err(CompileError::BreakOutsideLoop);
            }
        };

        self.emit_scope_cleanup(loop_depth);

        let jump = self.emit_jump(OpCode::Jump);

        self.loops
            .last_mut()
            .ok_or_else(|| {
                CompileError::InternalCompilerError("pile des boucles désynchronisée".to_string())
            })?
            .break_jumps
            .push(jump);

        Ok(())
    }

    pub(crate) fn compile_continue(&mut self) -> Result<(), CompileError> {
        let (continue_target, loop_depth) = match self.loops.last() {
            Some(loop_context) => (loop_context.continue_target, loop_context.scope_depth),

            None => {
                return Err(CompileError::ContinueOutsideLoop);
            }
        };

        self.emit_scope_cleanup(loop_depth);

        self.emit_loop(continue_target)?;

        Ok(())
    }
}

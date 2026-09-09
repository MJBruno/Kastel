use crate::bytecode::chunk::OpCode;
use crate::error::compile_error::CompileError;
use crate::frontend::ast::*;
use crate::runtime::value::Value;

use super::compiler::Compiler;
use super::variables::Global;

impl Compiler {
    pub(crate) fn register_export(&mut self, name: &str) -> Result<(), CompileError> {
        if self.exports.iter().any(|export| export == name) {
            return Err(CompileError::DuplicateExport(name.to_string()));
        }

        self.exports.push(name.to_string());

        Ok(())
    }

    // ============================================================
    // STATEMENTS
    // ============================================================

    pub fn compile_statement(&mut self, stmt: &Statement) -> Result<(), CompileError> {
        match stmt {
            Statement::Positioned {
                line,
                column,
                statement,
            } => {
                self.current_line = *line;
                self.current_column = *column;

                self.compile_statement(statement)?;
            }

            Statement::Expression { expression } => {
                self.compile_expression(expression)?;
                self.emit_opcode(OpCode::Pop);
            }

            Statement::Let {
                name,
                value,
                mutable,
            } => {
                self.compile_var(name, Some(value), *mutable)?;
            }

            Statement::Block(statements) => {
                self.begin_scope();

                for statement in statements {
                    self.compile_statement(statement)?;
                }

                self.end_scope();
            }

            Statement::Assignment { target, value } => match target {
                AssignmentTarget::Variable(name) => {
                    self.compile_expression(value)?;
                    self.compile_variable_set(name)?;
                    self.emit_opcode(OpCode::Pop);
                }

                AssignmentTarget::Index { object, index } => {
                    self.compile_expression(object)?;
                    self.compile_expression(index)?;
                    self.compile_expression(value)?;

                    self.emit_opcode(OpCode::SetIndex);
                }

                AssignmentTarget::Member { object, name } => {
                    self.compile_expression(object)?;
                    self.compile_expression(value)?;

                    let name_constant = self.identifier_constant(name)?;

                    self.emit_bytes(OpCode::SetProperty, name_constant);
                }
            },

            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.compile_if(condition, then_branch, else_branch.as_ref())?;
            }

            Statement::While { condition, body } => {
                self.compile_while(condition, body)?;
            }

            Statement::ForIn {
                variable,
                iterable,
                body,
            } => {
                self.compile_for_in(variable, iterable, body)?;
            }

            Statement::Match { value, arms } => {
                self.compile_match(value, arms)?;
            }

            // ========================================================
            // EXCEPTIONS
            // ========================================================
            Statement::Throw { value } => {
                self.compile_expression(value)?;
                self.emit_opcode(OpCode::Throw);
            }

            Statement::Try {
                try_body,
                catch_name,
                catch_body,
                finally_body,
            } => {
                self.compile_try(
                    try_body,
                    catch_name.as_deref(),
                    catch_body.as_deref(),
                    finally_body.as_deref(),
                )?;
            }

            Statement::Function { name, params, body } => {
                self.compile_function_statement(name, params, body)?;
            }

            Statement::Break => {
                self.compile_break()?;
            }

            Statement::Continue => {
                self.compile_continue()?;
            }

            Statement::Return { value } => {
                self.compile_return(value.as_ref())?;
            }

            Statement::Import { path } => {
                self.compile_import(path)?;
            }

            Statement::FromImport { module, items } => {
                self.compile_from_import(module, items)?;
            }

            Statement::Export { statement } => {
                self.compile_export(statement)?;
            }
        }

        Ok(())
    }

    // ============================================================
    // TRY / CATCH / FINALLY
    // ============================================================

fn compile_try(
    &mut self,
    try_body: &[Statement],
    catch_name: Option<&str>,
    catch_body: Option<&[Statement]>,
    finally_body: Option<&[Statement]>,
) -> Result<(), CompileError> {
    if catch_body.is_none() && finally_body.is_none() {
        return Err(CompileError::InternalCompilerError(
            "try doit avoir catch ou finally".to_string(),
        ));
    }

    if catch_name.is_some() && catch_body.is_none() {
        return Err(CompileError::InternalCompilerError(
            "catch_name present sans catch_body".to_string(),
        ));
    }

    // ========================================================
    // HANDLER
    // ========================================================

    self.emit_opcode(OpCode::PushExceptionHandler);

    let catch_operand = self.chunk.code.len();
    self.emit_u16(u16::MAX);

    let finally_operand = self.chunk.code.len();
    self.emit_u16(u16::MAX);

    // ========================================================
    // TRY
    // ========================================================

    if let Some(body) = finally_body {
        self.push_finally_block(body);
    }

    self.begin_scope();

    for statement in try_body {
        self.compile_statement(statement)?;
    }

    self.emit_opcode(OpCode::PopExceptionHandler);

    self.end_scope();

    let normal_end_jump = self.emit_jump(OpCode::Jump);

    // ========================================================
    // CATCH
    // ========================================================

    let mut catch_end_jump = None;

    let catch_ip;

    if let Some(body) = catch_body {
        catch_ip = self.chunk.code.len();

        self.begin_scope();

        if let Some(name) = catch_name {
            self.declare_existing_local(name, true)?;
        }

        for statement in body {
            self.compile_statement(statement)?;
        }

        self.end_scope();

        let jump = self.emit_jump(OpCode::Jump);
        catch_end_jump = Some(jump);
    } else {
        catch_ip = self.chunk.code.len();
    }

    // ========================================================
    // FINALLY
    // ========================================================

    if finally_body.is_some() {
        self.pop_finally_block();
    }

    let finally_ip = if let Some(body) = finally_body {
        let ip = self.chunk.code.len();

        self.begin_scope();

        for statement in body {
            self.compile_statement(statement)?;
        }

        self.end_scope();

        self.emit_opcode(OpCode::FinallyEnd);

        Some(ip)
    } else {
        None
    };

    // ========================================================
    // PATCH CATCH
    // ========================================================

    if catch_body.is_some() {
        self.patch_u16(catch_operand, catch_ip)?;
    } else {
        self.patch_u16(catch_operand, u16::MAX as usize)?;
    }

    // ========================================================
    // PATCH FINALLY
    // ========================================================

    if let Some(ip) = finally_ip {
        self.patch_u16(finally_operand, ip)?;
    } else {
        self.patch_u16(finally_operand, u16::MAX as usize)?;
    }

    // ========================================================
    // CATCH -> FINALLY / END
    // ========================================================

    if let Some(jump) = catch_end_jump {
        let target = finally_ip.unwrap_or(self.chunk.code.len());
        self.patch_jump_to(jump, target)?;
    }

    // ========================================================
    // TRY -> FINALLY / END
    // ========================================================

    let normal_target = finally_ip.unwrap_or(self.chunk.code.len());
    self.patch_jump_to(normal_end_jump, normal_target)?;

    Ok(())
}

    // ============================================================
    // PATCH ABSOLUTE JUMP
    // ============================================================

    fn patch_jump_to(&mut self, offset: usize, target: usize) -> Result<(), CompileError> {
        if offset + 1 >= self.chunk.code.len() {
            return Err(CompileError::InvalidJump);
        }

        let instruction_end = offset.checked_add(2).ok_or(CompileError::InvalidJump)?;

        if target < instruction_end {
            return Err(CompileError::InvalidJump);
        }

        let distance = target
            .checked_sub(instruction_end)
            .ok_or(CompileError::InvalidJump)?;

        if distance > u16::MAX as usize {
            return Err(CompileError::JumpTooLarge);
        }

        let distance = distance as u16;

        self.chunk.code[offset] = (distance >> 8) as u8;

        self.chunk.code[offset + 1] = (distance & 0xff) as u8;

        Ok(())
    }

    // ============================================================
    // MATCH
    // ============================================================

    pub(crate) fn compile_match(
        &mut self,
        value: &Expression,
        arms: &[MatchArm],
    ) -> Result<(), CompileError> {
        if arms.is_empty() {
            return Err(CompileError::InternalCompilerError(
                "match sans arm".to_string(),
            ));
        }

        self.begin_scope();

        let subject_name = format!("__match_subject_{}", self.context.borrow().locals.len());

        self.compile_local_var(&subject_name, Some(value), false)?;

        let subject_depth = self.scope_depth;

        let mut end_jumps = Vec::with_capacity(arms.len());

        for arm in arms {
            self.begin_scope();

            let pattern_false_jump = self.compile_match_pattern(&subject_name, &arm.pattern)?;

            self.emit_opcode(OpCode::Pop);

            if let Some(guard) = &arm.guard {
                self.compile_expression(guard)?;

                let guard_false_jump = self.emit_jump(OpCode::JumpIfFalse);

                self.emit_opcode(OpCode::Pop);

                for statement in &arm.body {
                    self.compile_statement(statement)?;
                }

                self.emit_scope_cleanup(subject_depth);

                let end_jump = self.emit_jump(OpCode::Jump);

                end_jumps.push(end_jump);

                self.patch_jump(guard_false_jump)?;

                self.emit_opcode(OpCode::Pop);

                self.emit_scope_cleanup(subject_depth);

                let next_arm_jump = self.emit_jump(OpCode::Jump);

                self.patch_jump(pattern_false_jump)?;

                self.emit_opcode(OpCode::Pop);

                self.patch_jump(next_arm_jump)?;
            } else {
                for statement in &arm.body {
                    self.compile_statement(statement)?;
                }

                self.emit_scope_cleanup(subject_depth);

                let end_jump = self.emit_jump(OpCode::Jump);

                end_jumps.push(end_jump);

                self.patch_jump(pattern_false_jump)?;

                self.emit_opcode(OpCode::Pop);
            }

            self.discard_scope();
        }

        for jump in end_jumps {
            self.patch_jump(jump)?;
        }

        self.end_scope();

        Ok(())
    }

    // ============================================================
    // MATCH PATTERN
    // ============================================================

    fn compile_match_pattern(
        &mut self,
        subject_name: &str,
        pattern: &Pattern,
    ) -> Result<usize, CompileError> {
        let subject = Expression::Variable(subject_name.to_string());

        let test_false_jump = self.compile_pattern_test_expression(&subject, pattern)?;

        self.emit_opcode(OpCode::Pop);

        self.compile_pattern_bindings(&subject, pattern)?;

        self.emit_opcode(OpCode::True);

        let final_false_jump = self.emit_jump(OpCode::JumpIfFalse);

        self.patch_jump(test_false_jump)?;

        Ok(final_false_jump)
    }

    // ============================================================
    // PATTERN TEST
    // ============================================================

    fn compile_pattern_test_expression(
        &mut self,
        expression: &Expression,
        pattern: &Pattern,
    ) -> Result<usize, CompileError> {
        match pattern {
            Pattern::Wildcard => {
                self.emit_opcode(OpCode::True);

                Ok(self.emit_jump(OpCode::JumpIfFalse))
            }

            Pattern::Binding(_) => {
                self.emit_opcode(OpCode::True);

                Ok(self.emit_jump(OpCode::JumpIfFalse))
            }

            Pattern::Literal(literal) => {
                self.compile_expression(expression)?;

                self.compile_literal_pattern(literal)?;

                self.emit_opcode(OpCode::Equal);

                Ok(self.emit_jump(OpCode::JumpIfFalse))
            }

            Pattern::Or(patterns) => self.compile_or_pattern_expression(expression, patterns),

            Pattern::Range {
                start,
                end,
                inclusive,
            } => self.compile_range_pattern_expression(expression, start, end, *inclusive),

            Pattern::Array(patterns) => self.compile_array_pattern_expression(expression, patterns),
        }
    }

    // ============================================================
    // LITERAL
    // ============================================================

    fn compile_literal_pattern(&mut self, literal: &Literal) -> Result<(), CompileError> {
        match literal {
            Literal::Integer(value) => {
                let constant = self.make_constant(Value::Integer(*value))?;

                self.emit_bytes(OpCode::Constant, constant);
            }

            Literal::Float(value) => {
                let constant = self.make_constant(Value::Float(*value))?;

                self.emit_bytes(OpCode::Constant, constant);
            }

            Literal::String(value) => {
                let constant = self.make_constant(Value::new_string(value.clone()))?;

                self.emit_bytes(OpCode::Constant, constant);
            }

            Literal::Bool(true) => {
                self.emit_opcode(OpCode::True);
            }

            Literal::Bool(false) => {
                self.emit_opcode(OpCode::False);
            }

            Literal::Nil => {
                self.emit_opcode(OpCode::Nil);
            }
        }

        Ok(())
    }

    // ============================================================
    // OR
    // ============================================================

    fn compile_or_pattern_expression(
        &mut self,
        expression: &Expression,
        patterns: &[Pattern],
    ) -> Result<usize, CompileError> {
        if patterns.is_empty() {
            return Err(CompileError::InternalCompilerError(
                "Pattern OR vide".to_string(),
            ));
        }

        if patterns.iter().any(Self::pattern_contains_binding) {
            return Err(CompileError::InternalCompilerError(
                "Binding directement dans un pattern OR non supporte".to_string(),
            ));
        }

        let mut success_jumps = Vec::new();

        for pattern in patterns.iter().take(patterns.len() - 1) {
            let false_jump = self.compile_pattern_test_expression(expression, pattern)?;

            self.emit_opcode(OpCode::Pop);

            self.emit_opcode(OpCode::True);

            let success_jump = self.emit_jump(OpCode::Jump);

            success_jumps.push(success_jump);

            self.patch_jump(false_jump)?;

            self.emit_opcode(OpCode::Pop);
        }

        let last_pattern = &patterns[patterns.len() - 1];

        let last_false_jump = self.compile_pattern_test_expression(expression, last_pattern)?;

        for success_jump in success_jumps {
            self.patch_jump(success_jump)?;
        }

        Ok(last_false_jump)
    }

    // ============================================================
    // RANGE
    // ============================================================

    fn compile_range_pattern_expression(
        &mut self,
        expression: &Expression,
        start: &Pattern,
        end: &Pattern,
        inclusive: bool,
    ) -> Result<usize, CompileError> {
        let start_literal = match start {
            Pattern::Literal(literal) => literal,

            _ => {
                return Err(CompileError::InternalCompilerError(
                    "Le debut du range doit etre un litteral".to_string(),
                ));
            }
        };

        let end_literal = match end {
            Pattern::Literal(literal) => literal,

            _ => {
                return Err(CompileError::InternalCompilerError(
                    "La fin du range doit etre un litteral".to_string(),
                ));
            }
        };

        self.compile_expression(expression)?;

        self.compile_literal_pattern(start_literal)?;

        self.emit_opcode(OpCode::Less);
        self.emit_opcode(OpCode::Not);

        let lower_false_jump = self.emit_jump(OpCode::JumpIfFalse);

        self.emit_opcode(OpCode::Pop);

        self.compile_expression(expression)?;

        self.compile_literal_pattern(end_literal)?;

        if inclusive {
            self.emit_opcode(OpCode::Greater);
            self.emit_opcode(OpCode::Not);
        } else {
            self.emit_opcode(OpCode::Less);
        }

        let upper_false_jump = self.emit_jump(OpCode::JumpIfFalse);

        let result_jump = self.emit_jump(OpCode::Jump);

        self.patch_jump(lower_false_jump)?;

        let lower_result_jump = self.emit_jump(OpCode::Jump);

        self.patch_jump(upper_false_jump)?;

        let upper_result_jump = self.emit_jump(OpCode::Jump);

        self.patch_jump(result_jump)?;

        self.patch_jump(lower_result_jump)?;

        self.patch_jump(upper_result_jump)?;

        Ok(self.emit_jump(OpCode::JumpIfFalse))
    }

    // ============================================================
    // ARRAY PATTERN
    // ============================================================

    fn compile_array_pattern_expression(
        &mut self,
        expression: &Expression,
        patterns: &[Pattern],
    ) -> Result<usize, CompileError> {
        self.compile_expression(expression)?;

        self.emit_opcode(OpCode::ArrayLength);

        let length_constant = self.make_constant(Value::Integer(patterns.len() as i64))?;

        self.emit_bytes(OpCode::Constant, length_constant);

        self.emit_opcode(OpCode::Equal);

        let length_false_jump = self.emit_jump(OpCode::JumpIfFalse);

        self.emit_opcode(OpCode::Pop);

        let mut element_false_jumps = Vec::new();

        for (index, pattern) in patterns.iter().enumerate() {
            let element_expression = Expression::Index {
                object: Box::new(expression.clone()),
                index: Box::new(Expression::Literal(Literal::Integer(index as i64))),
            };

            let false_jump = self.compile_pattern_test_expression(&element_expression, pattern)?;

            self.emit_opcode(OpCode::Pop);

            element_false_jumps.push(false_jump);
        }

        self.emit_opcode(OpCode::True);

        let success_jump = self.emit_jump(OpCode::Jump);

        self.patch_jump(length_false_jump)?;

        self.emit_opcode(OpCode::Pop);

        self.emit_opcode(OpCode::False);

        let length_result_jump = self.emit_jump(OpCode::Jump);

        let mut element_result_jumps = Vec::new();

        for false_jump in element_false_jumps {
            self.patch_jump(false_jump)?;

            self.emit_opcode(OpCode::Pop);

            self.emit_opcode(OpCode::False);

            let result_jump = self.emit_jump(OpCode::Jump);

            element_result_jumps.push(result_jump);
        }

        self.patch_jump(success_jump)?;

        self.patch_jump(length_result_jump)?;

        for jump in element_result_jumps {
            self.patch_jump(jump)?;
        }

        Ok(self.emit_jump(OpCode::JumpIfFalse))
    }

    // ============================================================
    // PATTERN BINDINGS
    // ============================================================

    fn compile_pattern_bindings(
        &mut self,
        expression: &Expression,
        pattern: &Pattern,
    ) -> Result<(), CompileError> {
        match pattern {
            Pattern::Wildcard | Pattern::Literal(_) | Pattern::Range { .. } => Ok(()),

            Pattern::Binding(name) => {
                self.compile_local_var(name, Some(expression), true)?;

                Ok(())
            }

            Pattern::Or(patterns) => {
                if patterns.iter().any(Self::pattern_contains_binding) {
                    return Err(CompileError::InternalCompilerError(
                        "Binding directement dans un pattern OR non supporte".to_string(),
                    ));
                }

                Ok(())
            }

            Pattern::Array(patterns) => {
                for (index, child) in patterns.iter().enumerate() {
                    let element_expression = Expression::Index {
                        object: Box::new(expression.clone()),
                        index: Box::new(Expression::Literal(Literal::Integer(index as i64))),
                    };

                    self.compile_pattern_bindings(&element_expression, child)?;
                }

                Ok(())
            }
        }
    }

    // ============================================================
    // HELPERS
    // ============================================================

    fn pattern_contains_binding(pattern: &Pattern) -> bool {
        match pattern {
            Pattern::Binding(_) => true,

            Pattern::Wildcard | Pattern::Literal(_) => false,

            Pattern::Or(patterns) => patterns.iter().any(Self::pattern_contains_binding),

            Pattern::Range { start, end, .. } => {
                Self::pattern_contains_binding(start) || Self::pattern_contains_binding(end)
            }

            Pattern::Array(patterns) => patterns.iter().any(Self::pattern_contains_binding),
        }
    }

    // ============================================================
    // IMPORT
    // ============================================================

    pub(crate) fn compile_from_import(
        &mut self,
        module: &ModulePath,
        items: &[ImportItem],
    ) -> Result<(), CompileError> {
        if module.parts.is_empty() || items.is_empty() {
            return Err(CompileError::InvalidImport);
        }

        let module_name = module.parts.join(".");

        for item in items {
            let binding_name = item.alias.as_deref().unwrap_or(&item.name);

            if self.globals.borrow().contains_key(binding_name) {
                return Err(CompileError::VariableAlreadyDeclared(
                    binding_name.to_string(),
                ));
            }

            let module_constant = self.make_constant(Value::new_string(module_name.clone()))?;

            self.emit_bytes(OpCode::Import, module_constant);

            let property_constant = self.identifier_constant(&item.name)?;

            self.emit_bytes(OpCode::GetProperty, property_constant);

            let binding_constant = self.identifier_constant(binding_name)?;

            self.emit_bytes(OpCode::DefineGlobal, binding_constant);

            self.globals.borrow_mut().insert(
                binding_name.to_string(),
                Global {
                    constant: binding_constant,
                    mutable: false,
                },
            );
        }

        Ok(())
    }

    pub(crate) fn compile_import(&mut self, path: &[String]) -> Result<(), CompileError> {
        if path.is_empty() {
            return Err(CompileError::InvalidImport);
        }

        let module_name = path.join(".");

        let binding_name = path.first().ok_or(CompileError::InvalidImport)?;

        if self.imported_modules.contains(binding_name) {
            return Ok(());
        }

        if self.globals.borrow().contains_key(binding_name) {
            return Err(CompileError::VariableAlreadyDeclared(binding_name.clone()));
        }

        let module_constant = self.make_constant(Value::new_string(module_name))?;

        self.emit_bytes(OpCode::Import, module_constant);

        let name_constant = self.identifier_constant(binding_name)?;

        self.emit_bytes(OpCode::DefineGlobal, name_constant);

        self.globals.borrow_mut().insert(
            binding_name.clone(),
            Global {
                constant: name_constant,
                mutable: false,
            },
        );

        self.imported_modules.insert(binding_name.clone());

        Ok(())
    }

    // ============================================================
    // EXPORT
    // ============================================================

    pub(crate) fn compile_export(&mut self, statement: &Statement) -> Result<(), CompileError> {
        if self.in_function || self.scope_depth != 0 {
            return Err(CompileError::InvalidExport);
        }

        match statement {
            Statement::Let { name, .. } => {
                self.register_export(name)?;

                self.compile_statement(statement)?;
            }

            Statement::Function { name, .. } => {
                self.register_export(name)?;

                self.compile_statement(statement)?;
            }

            _ => {
                return Err(CompileError::InvalidExport);
            }
        }

        Ok(())
    }
}

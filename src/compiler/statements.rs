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

        /*
         * Le sujet est évalué une seule fois.
         *
         * Scope:
         *
         *   match scope
         *       └── subject
         */
        self.begin_scope();

        let subject_name = format!("__match_subject_{}", self.context.borrow().locals.len());

        self.compile_local_var(&subject_name, Some(value), false)?;

        /*
         * Profondeur du scope du sujet.
         */
        let subject_depth = self.scope_depth;

        let mut end_jumps = Vec::with_capacity(arms.len());

        /*
         * Compile chaque arm.
         */
        for arm in arms {
            /*
             * Scope propre à l'arm.
             *
             * Les bindings vivent ici.
             */
            self.begin_scope();

            let arm_depth = self.scope_depth;

            /*
             * Compile le pattern.
             *
             * Convention :
             *
             *   succès -> true sur la pile
             *   échec  -> false sur la pile
             */
            let pattern_false_jump = self.compile_match_pattern(&subject_name, &arm.pattern)?;

            /*
             * ========================================================
             * PATTERN SUCCESS
             * ========================================================
             *
             * JumpIfFalse laisse true sur la pile.
             */
            self.emit_opcode(OpCode::Pop);

            /*
             * ========================================================
             * GUARD
             * ========================================================
             */
            if let Some(guard) = &arm.guard {
                self.compile_expression(guard)?;

                let guard_false_jump = self.emit_jump(OpCode::JumpIfFalse);

                /*
                 * Guard true.
                 *
                 * Retire le booléen.
                 */
                self.emit_opcode(OpCode::Pop);

                /*
                 * Body.
                 */
                for statement in &arm.body {
                    self.compile_statement(statement)?;
                }

                /*
                 * Nettoyage des bindings de l'arm.
                 */
                self.emit_scope_cleanup(subject_depth);

                /*
                 * Aller à la fin du match.
                 */
                let end_jump = self.emit_jump(OpCode::Jump);

                end_jumps.push(end_jump);

                /*
                 * ====================================================
                 * GUARD FALSE
                 * ====================================================
                 */
                self.patch_jump(guard_false_jump)?;

                /*
                 * JumpIfFalse conserve false.
                 */
                self.emit_opcode(OpCode::Pop);

                /*
                 * Nettoyer les bindings.
                 */
                self.emit_scope_cleanup(subject_depth);

                /*
                 * Aller au prochain arm.
                 */
                let next_arm_jump = self.emit_jump(OpCode::Jump);

                /*
                 * ====================================================
                 * PATTERN FALSE
                 * ====================================================
                 *
                 * Un pattern refutable arrive ici.
                 */
                self.patch_jump(pattern_false_jump)?;

                /*
                 * Le résultat false est encore sur la pile.
                 */
                self.emit_opcode(OpCode::Pop);

                /*
                 * Le pattern était faux : passer au prochain arm.
                 */
                self.patch_jump(next_arm_jump)?;
            } else {
                /*
                 * ====================================================
                 * NO GUARD
                 * ====================================================
                 */

                for statement in &arm.body {
                    self.compile_statement(statement)?;
                }

                /*
                 * Nettoyage des bindings.
                 */
                self.emit_scope_cleanup(subject_depth);

                /*
                 * Arm réussi -> fin du match.
                 */
                let end_jump = self.emit_jump(OpCode::Jump);

                end_jumps.push(end_jump);

                /*
                 * ====================================================
                 * PATTERN FALSE
                 * ====================================================
                 */
                self.patch_jump(pattern_false_jump)?;

                /*
                 * JumpIfFalse laisse false.
                 */
                self.emit_opcode(OpCode::Pop);
            }

            /*
             * IMPORTANT :
             *
             * Les Pop de cleanup ont déjà été générés dans chaque
             * chemin d'exécution.
             *
             * On retire donc maintenant les locals du scope
             * uniquement côté compilateur.
             */
            self.discard_scope();

            /*
             * Vérification logique :
             *
             * arm_depth est conservé pour documenter l'invariant du scope.
             */
            let _ = arm_depth;
        }

        /*
         * ============================================================
         * FIN DU MATCH
         * ============================================================
         *
         * Tous les end_jumps arrivent ici.
         */
        for jump in end_jumps {
            self.patch_jump(jump)?;
        }

        /*
         * Supprime le subject.
         *
         * end_scope() émet exactement un Pop pour le local subject.
         */
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
        match pattern {
            // --------------------------------------------------------
            // _
            // --------------------------------------------------------
            Pattern::Wildcard => {
                /*
                 * Wildcard = toujours vrai.
                 */
                self.emit_opcode(OpCode::True);

                let jump = self.emit_jump(OpCode::JumpIfFalse);

                Ok(jump)
            }

            // --------------------------------------------------------
            // x
            // --------------------------------------------------------
            Pattern::Binding(name) => {
                /*
                 * Le binding récupère le sujet :
                 *
                 *     match value {
                 *         x => println(x),
                 *     }
                 *
                 * devient :
                 *
                 *     let x = value;
                 */
                let expression = Expression::Variable(subject_name.to_string());

                self.compile_local_var(name, Some(&expression), true)?;

                /*
                 * Un binding simple est irrefutable.
                 */
                self.emit_opcode(OpCode::True);

                let jump = self.emit_jump(OpCode::JumpIfFalse);

                Ok(jump)
            }

            // --------------------------------------------------------
            // literal
            // --------------------------------------------------------
            Pattern::Literal(literal) => {
                self.compile_variable_get(subject_name)?;

                self.compile_literal_pattern(literal)?;

                self.emit_opcode(OpCode::Equal);

                let jump = self.emit_jump(OpCode::JumpIfFalse);

                Ok(jump)
            }

            // --------------------------------------------------------
            // OR
            // --------------------------------------------------------
            Pattern::Or(patterns) => self.compile_or_pattern(subject_name, patterns),

            // --------------------------------------------------------
            // RANGE
            // --------------------------------------------------------
            Pattern::Range {
                start,
                end,
                inclusive,
            } => self.compile_range_pattern(subject_name, start, end, *inclusive),

            // --------------------------------------------------------
            // ARRAY
            // --------------------------------------------------------
            Pattern::Array(patterns) => self.compile_array_pattern(subject_name, patterns),
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


fn compile_or_pattern(
    &mut self,
    subject_name: &str,
    patterns: &[Pattern],
) -> Result<usize, CompileError> {
    if patterns.is_empty() {
        return Err(
            CompileError::InternalCompilerError(
                "Pattern OR vide".to_string(),
            ),
        );
    }

    /*
     * Pour l'instant :
     *
     *     1 | 7 | 10
     *
     * est supporté.
     *
     * Les bindings dans un OR seront ajoutés ensuite.
     */
    for pattern in patterns {
        if Self::pattern_contains_binding(pattern) {
            return Err(
                CompileError::InternalCompilerError(
                    "Binding dans un pattern OR non encore supporté"
                        .to_string(),
                ),
            );
        }

        match pattern {
            Pattern::Literal(_) | Pattern::Wildcard => {}

            _ => {
                return Err(
                    CompileError::InternalCompilerError(
                        "Ce type de pattern n'est pas encore supporté dans OR"
                            .to_string(),
                    ),
                );
            }
        }
    }

    /*
     * Les Jump qui correspondent à un succès des alternatives
     * précédentes seront tous redirigés vers le même point final.
     */
    let mut success_jumps = Vec::new();

    /*
     * Tous les patterns sauf le dernier.
     *
     * Exemple :
     *
     *     1 | 7 | 10
     *
     * produit :
     *
     *     test 1
     *     false -> test 7
     *     true  -> success
     *
     *     test 7
     *     false -> test 10
     *     true  -> success
     */
    for pattern in patterns.iter().take(patterns.len() - 1) {
        let false_jump =
            self.compile_match_pattern(
                subject_name,
                pattern,
            )?;

        /*
         * Pattern réussi :
         *
         * JumpIfFalse a laissé true.
         */
        self.emit_opcode(OpCode::Pop);

        /*
         * Produire le résultat true du OR.
         *
         * Ce true sera utilisé par compile_match().
         */
        self.emit_opcode(OpCode::True);

        /*
         * Aller au résultat commun.
         */
        let success_jump =
            self.emit_jump(OpCode::Jump);

        success_jumps.push(success_jump);

        /*
         * Pattern échoué :
         *
         * JumpIfFalse a laissé false.
         */
        self.patch_jump(false_jump)?;

        /*
         * Supprimer false avant de tester l'alternative suivante.
         */
        self.emit_opcode(OpCode::Pop);
    }

    /*
     * Dernier pattern.
     *
     * Son résultat devient directement le résultat du OR :
     *
     *     true  -> match arm
     *     false -> prochain arm
     */
    let last_false_jump =
        self.compile_match_pattern(
            subject_name,
            patterns
                .last()
                .expect("patterns non vide"),
        )?;

    /*
     * Les succès des alternatives précédentes arrivent ici.
     *
     * Ils ont déjà placé true sur la pile.
     *
     * Le dernier pattern arrive lui aussi ici avec true sur la pile
     * lorsqu'il réussit.
     */
    for success_jump in success_jumps {
        self.patch_jump(success_jump)?;
    }

    /*
     * IMPORTANT :
     *
     * Le dernier pattern possède déjà son JumpIfFalse.
     *
     * C'est exactement le jump que compile_match() doit utiliser
     * pour passer à l'arm suivant.
     */
    Ok(last_false_jump)
}



    // ============================================================
    // RANGE
    // ============================================================

    fn compile_range_pattern(
        &mut self,
        subject_name: &str,
        start: &Pattern,
        end: &Pattern,
        inclusive: bool,
    ) -> Result<usize, CompileError> {
        let start_literal = match start {
            Pattern::Literal(literal) => literal,
            _ => {
                return Err(CompileError::InternalCompilerError(
                    "Le début du range doit être un littéral".to_string(),
                ));
            }
        };

        let end_literal = match end {
            Pattern::Literal(literal) => literal,
            _ => {
                return Err(CompileError::InternalCompilerError(
                    "La fin du range doit être un littéral".to_string(),
                ));
            }
        };

        /*
         * subject >= start
         */
        self.compile_variable_get(subject_name)?;
        self.compile_literal_pattern(start_literal)?;

        self.emit_opcode(OpCode::Less);
        self.emit_opcode(OpCode::Not);

        /*
         * Si false :
         *
         *     subject < start
         *
         * On garde false sur la pile et le match peut passer
         * directement à l'arm suivant.
         */
        let lower_false_jump = self.emit_jump(OpCode::JumpIfFalse);

        /*
         * Le premier test est vrai.
         */
        self.emit_opcode(OpCode::Pop);

        /*
         * subject < end
         * ou
         * subject <= end
         */
        self.compile_variable_get(subject_name)?;
        self.compile_literal_pattern(end_literal)?;

        if inclusive {
            /*
             * <=
             */
            self.emit_opcode(OpCode::Greater);
            self.emit_opcode(OpCode::Not);
        } else {
            /*
             * <
             */
            self.emit_opcode(OpCode::Less);
        }

        /*
         * Le résultat de ce deuxième test devient directement
         * le résultat du pattern.
         */
        let upper_false_jump = self.emit_jump(OpCode::JumpIfFalse);

        /*
         * True :
         *
         * ne rien enlever.
         *
         * compile_match() exécutera son Pop sur le chemin succès.
         */
        let result_jump = self.emit_jump(OpCode::Jump);

        /*
         * ------------------------------------------------------------
         * PREMIER TEST FAUX
         * ------------------------------------------------------------
         */
        self.patch_jump(lower_false_jump)?;

        /*
         * false est déjà sur la pile.
         *
         * C'est le résultat du pattern.
         */
        let lower_result_jump = self.emit_jump(OpCode::Jump);

        /*
         * ------------------------------------------------------------
         * DEUXIÈME TEST FAUX
         * ------------------------------------------------------------
         */
        self.patch_jump(upper_false_jump)?;

        /*
         * false est déjà sur la pile.
         */
        let upper_result_jump = self.emit_jump(OpCode::Jump);

        /*
         * ------------------------------------------------------------
         * FIN
         * ------------------------------------------------------------
         */
        self.patch_jump(result_jump)?;
        self.patch_jump(lower_result_jump)?;
        self.patch_jump(upper_result_jump)?;

        /*
         * On place le JumpIfFalse commun attendu par compile_match().
         */
        Ok(self.emit_jump(OpCode::JumpIfFalse))
    }

    // ============================================================
    // ARRAY
    // ============================================================

    fn compile_array_pattern(
        &mut self,
        _subject_name: &str,
        _patterns: &[Pattern],
    ) -> Result<usize, CompileError> {
        Err(CompileError::InternalCompilerError(
            "Pattern tableau non encore activé".to_string(),
        ))
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

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
    //                      STATEMENTS
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

                // L'expression-statement ignore sa valeur : il faut la dépiler,
                // sinon elle s'accumule et décale l'index de toutes les
                // variables locales déclarées ensuite dans le même scope.
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

                    // SetLocal/SetGlobal/SetUpvalue laissent une copie de la
                    // valeur assignée sur la pile (pour un futur usage en tant
                    // qu'expression) : il faut la dépiler ici, sinon même bug
                    // de désynchronisation des slots locaux qu'avec
                    // Statement::Expression.
                    self.emit_opcode(OpCode::Pop);
                }

                AssignmentTarget::Index { object, index } => {
                    self.compile_expression(object)?;
                    self.compile_expression(index)?;
                    self.compile_expression(value)?;

                    // SetIndex consomme les 3 valeurs et ne repousse rien :
                    // la pile est déjà équilibrée, pas de Pop supplémentaire.
                    self.emit_opcode(OpCode::SetIndex);
                }

                AssignmentTarget::Member { object, name } => {
                    self.compile_expression(object)?;
                    self.compile_expression(value)?;

                    let name_constant = self.identifier_constant(name)?;

                    // Même convention que SetIndex : SetProperty consomme
                    // l'objet et la valeur sans rien repousser.
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
    //                      MODULES : IMPORT / EXPORT
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

            // import module
            let module_constant = self.make_constant(Value::new_string(module_name.clone()))?;

            self.emit_bytes(OpCode::Import, module_constant);

            // module.item
            let property_constant = self.identifier_constant(&item.name)?;

            self.emit_bytes(OpCode::GetProperty, property_constant);

            // define alias/name
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

    pub(crate) fn compile_export(&mut self, statement: &Statement) -> Result<(), CompileError> {
        //Evite l'export dans un function ou objet
        //      function outer() {
        //          export let x = 10;
        //      }
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

    pub(crate) fn compile_match(
        &mut self,
        value: &Expression,
        arms: &[MatchArm],
    ) -> Result<(), CompileError> {
        /* * Le sujet du match est évalué une seule fois. * * On le place dans une variable locale temporaire afin de * pouvoir le relire pour chaque arm sans ajouter de DUP au VM. */
        self.begin_scope();
        let temporary_name = format!("__match_{}", self.context.borrow().locals.len());
        self.compile_local_var(&temporary_name, Some(value), false)?;
        let mut end_jumps = Vec::with_capacity(arms.len());
        for arm in arms {
            let next_arm_jump = self.compile_match_pattern(&temporary_name, &arm.pattern)?; /* * Le résultat du test est présent sur la pile. * * JumpIfFalse conserve la condition sur la pile : * on la retire donc avant de compiler le corps. */
            self.emit_opcode(OpCode::Pop);
            for statement in &arm.body {
                self.compile_statement(statement)?;
            } /* * Le body est terminé : ne pas tomber dans l'arm suivant. */
            let end_jump = self.emit_jump(OpCode::Jump);
            end_jumps.push(end_jump); /* * Ici on arrive seulement lorsque le pattern précédent * n'a pas correspondu. * * Le booléen de JumpIfFalse est encore sur la pile. */
            self.patch_jump(next_arm_jump)?;
            self.emit_opcode(OpCode::Pop);
        } /* * Aucun arm ne correspond : * * À cette étape, on ne force pas encore l'exhaustivité statique * Rust. Le comportement sera complété avec le runtime match error. */
        for jump in end_jumps {
            self.patch_jump(jump)?;
        }
        self.end_scope();
        Ok(())
    }
    fn compile_match_pattern(
        &mut self,
        value_name: &str,
        pattern: &Pattern,
    ) -> Result<usize, CompileError> {
        match pattern {
            Pattern::Wildcard => {
                /* * Le wildcard correspond toujours. */
                self.emit_opcode(OpCode::True); /* * Impossible d'avoir un prochain arm réellement * nécessaire puisque `_` capture tout. * * On émet tout de même un JumpIfFalse pour conserver * une représentation uniforme. */
                let jump = self.emit_jump(OpCode::JumpIfFalse);
                Ok(jump)
            }
            Pattern::Literal(literal) => {
                /* * Stack : * * [ subject ] * * GetLocal ajoute : * * [ subject, pattern ] */
                self.compile_variable_get(value_name)?;
                self.compile_literal_pattern(literal)?; /* * Equal : * * [ subject, pattern ] * ↓ * [ bool ] */
                self.emit_opcode(OpCode::Equal); /* * Résultat : * * true -> body * false -> arm suivant */
                let jump = self.emit_jump(OpCode::JumpIfFalse);
                Ok(jump)
            }
            Pattern::Or(patterns) => {
                if patterns.is_empty() {
                    return Err(CompileError::InternalCompilerError(
                        "Pattern OR vide".to_string(),
                    ));
                } /* * Chaque alternative doit pouvoir réussir * indépendamment. * * Pour cette étape, on génère simplement : * * test1 * if true -> success * test2 * if true -> success * ... * * Le résultat final est un booléen unique. */
                let mut success_jumps = Vec::new();
                for (index, pattern) in patterns.iter().enumerate() {
                    match pattern {
                        Pattern::Literal(literal) => {
                            self.compile_variable_get(value_name)?;
                            self.compile_literal_pattern(literal)?;
                            self.emit_opcode(OpCode::Equal);
                            let false_jump = self.emit_jump(OpCode::JumpIfFalse);
                            self.emit_opcode(OpCode::Pop);
                            if index + 1 < patterns.len() {
                                let success_jump = self.emit_jump(OpCode::Jump);
                                success_jumps.push(success_jump);
                                self.patch_jump(false_jump)?;
                            } else {
                                /* * Dernière alternative : * conserver son résultat pour le * JumpIfFalse du match principal. */
                                self.patch_jump(false_jump)?;
                            }
                        }
                        Pattern::Wildcard => {
                            self.emit_opcode(OpCode::True);
                            let success_jump = self.emit_jump(OpCode::JumpIfFalse);
                            success_jumps.push(success_jump);
                        }
                        _ => {
                            return Err(CompileError::InternalCompilerError(
                                "Pattern imbriqué non supporté à cette étape".to_string(),
                            ));
                        }
                    }
                } /* * Tous les jumps de succès convergent ici. * * Pour cette première étape, on termine sur un booléen. */
                let jump = self.emit_jump(OpCode::JumpIfFalse);
                for success_jump in success_jumps {
                    self.patch_jump(success_jump)?;
                }
                Ok(jump)
            }
            Pattern::Range { .. } => Err(CompileError::InternalCompilerError(
                "Pattern range non encore compilé".to_string(),
            )),
            Pattern::Binding(_) => Err(CompileError::InternalCompilerError(
                "Binding de pattern non encore compilé".to_string(),
            )),
            Pattern::Array(_) => Err(CompileError::InternalCompilerError(
                "Pattern tableau non encore compilé".to_string(),
            )),
        }
    }
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
}

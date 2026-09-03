use crate::bytecode::chunk::OpCode;
// use crate::compiler::statements;
use crate::error::compile_error::CompileError;
use crate::frontend::ast::*;
use crate::runtime::value::Value;

use super::compiler::Compiler;
use super::variables::Global;

impl Compiler {
    // ============================================================
    //                      STATEMENTS
    // ============================================================

    pub fn compile_statement(&mut self, stmt: &Statement) -> Result<(), CompileError> {
        match stmt {
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

            Statement::Positioned {
              
                statement,
            } => {
                self.compile_statement(statement)?;
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
            let module_constant = self.make_constant(Value::String(module_name.clone()))?;

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

        // Pour :
        // import math;
        //
        // le nom disponible dans le scope est "math".
        let binding_name = path.first().ok_or(CompileError::InvalidImport)?;

        if self.globals.borrow().contains_key(binding_name) {
            return Err(CompileError::VariableAlreadyDeclared(binding_name.clone()));
        }

        let module_constant = self.make_constant(Value::String(module_name))?;

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

        Ok(())
    }

    pub(crate) fn compile_export(&mut self, statement: &Statement) -> Result<(), CompileError> {
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

    pub(crate) fn register_export(&mut self, name: &str) -> Result<(), CompileError> {
        if self.exports.iter().any(|export| export == name) {
            return Err(CompileError::DuplicateExport(name.to_string()));
        }

        self.exports.push(name.to_string());

        Ok(())
    }
}

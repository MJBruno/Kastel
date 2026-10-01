use crate::bytecode::chunk::OpCode;
use crate::error::compile_error::CompileError;
use crate::frontend::ast::*;
use crate::runtime::function::Function;
use crate::runtime::value::Value;

use super::compiler::Compiler;
use super::module_types::ImportedType;
use super::type_checker::{TypeCheckContext, TypeChecker};
use super::variables::{Global, VariableLocation};

#[allow(dead_code)]
impl Compiler {
    /// `true` si `path` (segments pointés, ex. `["m", "Person"]`) désigne un
    /// alias de TYPE pur — aucune valeur à l'exécution (voir
    /// `ModuleTypeInterface::type_aliases`). `compile_import` et
    /// `compile_from_import` n'émettent alors aucun bytecode pour ce nom :
    /// en émettre aurait fait échouer le programme à l'exécution en tentant
    /// de lire une propriété absente du module (rien n'y définit ce nom).
    ///
    /// `false` sans `self.type_context` (imports non résolus dans ce
    /// contexte de compilation) : le comportement d'avant reste alors
    /// inchangé.
    fn is_type_only_import(&self, path: &[String]) -> bool {
        let Some(context) = &self.type_context else {
            return false;
        };

        matches!(
            context
                .module_loader
                .resolve_import(&context.current_module, path),
            Ok(ImportedType::TypeAlias { .. })
        )
    }

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
                ..
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
                    let local_local_optimized = match value {
                        Expression::Binary {
                            left,
                            operator: BinaryOp::Add,
                            right,
                            ..
                        } => match (left.as_ref(), right.as_ref()) {
                            (Expression::Variable(left_name), Expression::Variable(right_name))
                                if left_name == name =>
                            {
                                let left_location = self.resolve_variable(left_name)?;
                                let right_location = self.resolve_variable(right_name)?;

                                match (left_location, right_location) {
                                    (
                                        VariableLocation::Local(left_slot),
                                        VariableLocation::Local(right_slot),
                                    ) => {
                                        if let Some(false) =
                                            self.context.borrow().locals.is_mutable(name)?
                                        {
                                            return Err(CompileError::AssignmentToConstant(
                                                name.to_string(),
                                            ));
                                        }

                                        self.emit_bytes(OpCode::AddLocalLocal, left_slot as u8);
                                        self.emit_byte(right_slot as u8);
                                        true
                                    }

                                    _ => false,
                                }
                            }

                            _ => false,
                        },

                        _ => false,
                    };

                    if local_local_optimized {
                        return Ok(());
                    }

                    /*
                     * Fast path :
                     *
                     *     i = i + 1
                     *
                     * devient :
                     *
                     *     AddLocalConst <slot> <constant>
                     */
                    let optimized = match value {
                        Expression::Binary {
                            left,
                            operator: BinaryOp::Add,
                            right,
                            ..
                        } => match left.as_ref() {
                            Expression::Variable(left_name) if left_name == name => {
                                let literal = match right.as_ref() {
                                    Expression::Literal(Literal::Integer(value)) => {
                                        Some(Value::Integer(*value))
                                    }

                                    Expression::Literal(Literal::Float(value)) => {
                                        Some(Value::Float(*value))
                                    }

                                    _ => None,
                                };

                                match literal {
                                    Some(constant_value) => match self.resolve_variable(name)? {
                                        VariableLocation::Local(slot) => {
                                            if let Some(false) =
                                                self.context.borrow().locals.is_mutable(name)?
                                            {
                                                return Err(CompileError::AssignmentToConstant(
                                                    name.to_string(),
                                                ));
                                            }

                                            let constant = self.make_constant(constant_value)?;

                                            // Super-instruction : opérande constante sur un
                                            // octet uniquement. Au-delà de 255, on retombe
                                            // sur la compilation générale.
                                            match u8::try_from(constant) {
                                                Ok(narrow) => {
                                                    self.emit_bytes(
                                                        OpCode::AddLocalConst,
                                                        slot as u8,
                                                    );

                                                    self.emit_byte(narrow);

                                                    true
                                                }

                                                Err(_) => false,
                                            }
                                        }

                                        _ => false,
                                    },

                                    None => false,
                                }
                            }

                            _ => false,
                        },

                        _ => false,
                    };

                    if !optimized {
                        self.compile_expression(value)?;
                        self.compile_variable_set(name)?;

                        // `SetLocal` / `SetGlobal` / `SetUpvalue` laissent la valeur
                        // affectée sur la pile (peek, pas pop) afin qu'une affectation
                        // puisse être utilisée comme une expression. Ici on est dans
                        // une instruction (`sum += x;`), donc cette valeur est inutilisée
                        // et doit être retirée, sans quoi elle s'accumule sur la pile à
                        // chaque itération d'une boucle et désynchronise les slots des
                        // variables locales déclarées ensuite (voir `for..in` + `continue`).
                        self.emit_opcode(OpCode::Pop);
                    }
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

                    self.emit_constant_op(OpCode::SetProperty, name_constant);
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
                catch_type,
                catch_body,
                finally_body,
            } => {
                self.compile_try(
                    try_body,
                    catch_name.as_deref(),
                    catch_type.as_ref(),
                    catch_body.as_deref(),
                    finally_body.as_deref(),
                )?;
            }

            Statement::Class {
                name,
                bases,
                fields,
                methods,
                ..
            } => {
                self.compile_class(name, bases, fields, methods)?;
            }

            Statement::Interface {
                name,
                bases,
                methods,
                ..
            } => {
                self.compile_interface(name, bases, methods)?;
            }

            Statement::Enum {
                name,
                variants,
                methods,
                ..
            } => {
                self.compile_enum(name, variants, methods)?;
            }

            // Un alias de type n'existe qu'à la compilation (vérificateur) :
            // aucun code à générer.
            Statement::TypeAlias { .. } => {}

            Statement::Function {
                name,
                params,
                body,
                is_async,
                ..
            } => {
                self.compile_function_statement(name, params, body, *is_async)?;
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
    // CLASS
    // ============================================================

    pub(crate) fn compile_class(
        &mut self,
        name: &str,
        bases: &[TypeExpr],
        fields: &[ClassField],
        methods: &[FunctionMethod],
    ) -> Result<(), CompileError> {
        if bases.len() > u8::MAX as usize {
            return Err(CompileError::TooManyObjectFields);
        }

        // Méthodes D'INSTANCE (appelées sur une instance, avec `this`) et
        // méthodes STATIQUES (appelées sur la classe elle-même, sans
        // receveur) sont stockées séparément par la VM : voir `op_class`.
        let instance_methods: Vec<&FunctionMethod> =
            methods.iter().filter(|method| !method.is_static).collect();
        let static_methods: Vec<&FunctionMethod> =
            methods.iter().filter(|method| method.is_static).collect();

        // Champs D'INSTANCE (une valeur par instance, initialisée via
        // `__fields_<Classe>`, voir `desugar_field_initializers`) et champs
        // STATIQUES (une seule valeur, portée par la classe).
        let static_fields: Vec<&ClassField> =
            fields.iter().filter(|field| field.is_static).collect();

        if instance_methods.len() > u8::MAX as usize
            || static_methods.len() > u8::MAX as usize
            || static_fields.len() > u8::MAX as usize
        {
            return Err(CompileError::TooManyObjectFields);
        }

        // Membres `protected` ou `private` (champs ET méthodes, statiques ou
        // non), sans doublon. La VM enregistre leur visibilité pour refaire
        // le contrôle lorsque le type statique est dynamique.
        let mut restricted_members: Vec<(&str, Visibility)> = Vec::new();

        let declared = fields
            .iter()
            .map(|field| (field.name.as_str(), field.visibility))
            .chain(
                methods
                    .iter()
                    .map(|method| (method.name.as_str(), method.visibility)),
            );

        for (member, visibility) in declared {
            if visibility != Visibility::Public
                && !restricted_members.iter().any(|(name, _)| *name == member)
            {
                restricted_members.push((member, visibility));
            }
        }

        if restricted_members.len() > u8::MAX as usize {
            return Err(CompileError::TooManyObjectFields);
        }

        // Si `name` a déjà été pré-déclaré par predeclare_global_function
        // (cas normal pour toute classe globale), ce n'est pas une vraie
        // redéclaration : on ne rejette que les collisions avec un nom
        // global qui existait déjà pour une AUTRE raison.
        if !self.in_function
            && self.scope_depth == 0
            && !self.predeclared_functions.contains(name)
            && let Some(global) = self.globals.borrow().get(name)
            && !global.native
        {
            return Err(CompileError::VariableAlreadyDeclared(name.to_string()));
        }
        let runtime_bases: Vec<&TypeExpr> = bases
            .iter()
            .filter(|base| {
                let Some(name) = Self::type_expr_name(base) else {
                    return false;
                };

                crate::compiler::capability::Capability::from_name(&name).is_none()
                    && !matches!(name.as_str(), "Iterator" | "Iterable")
            })
            .collect();

        for base in &runtime_bases {
            let base_name = Self::type_expr_name(base).ok_or_else(|| {
                CompileError::InternalCompilerError(
                    "Une interface générique doit désigner un type nommé".to_string(),
                )
            })?;
            self.compile_variable_get(&base_name)?;
        }

        let class_name_constant = self.identifier_constant(name)?;

        self.emit_constant_op(OpCode::Constant, class_name_constant);

        for method in &instance_methods {
            let method_name_constant = self.identifier_constant(&method.name)?;

            self.emit_constant_op(OpCode::Constant, method_name_constant);

            let function = self.compile_method(&method.name, &method.params, &method.body)?;

            let function_constant =
                self.make_constant(Value::new_function(std::rc::Rc::new(function.clone())))?;

            self.emit_closure(function_constant, &function.upvalues);
        }

        for method in &static_methods {
            let method_name_constant = self.identifier_constant(&method.name)?;

            self.emit_constant_op(OpCode::Constant, method_name_constant);

            let function =
                self.compile_static_method(&method.name, &method.params, &method.body)?;

            let function_constant =
                self.make_constant(Value::new_function(std::rc::Rc::new(function.clone())))?;

            self.emit_closure(function_constant, &function.upvalues);
        }

        // Champs statiques : nom, puis valeur initiale (expression évaluée
        // UNE SEULE FOIS, ici, à la déclaration de la classe — pas de `this`
        // puisqu'il n'y a pas d'instance). Sans initialiseur, la valeur est
        // `None`, comme une variable dynamique jamais assignée.
        for field in &static_fields {
            let field_name_constant = self.identifier_constant(&field.name)?;

            self.emit_constant_op(OpCode::Constant, field_name_constant);

            match &field.initializer {
                Some(initializer) => self.compile_expression(initializer)?,
                None => self.emit_opcode(OpCode::None),
            }
        }

        // Noms et tags des membres restreints, empilés en dernier.
        // 1 = protected, 2 = private.
        for (member, visibility) in &restricted_members {
            let member_constant = self.identifier_constant(member)?;
            self.emit_constant_op(OpCode::Constant, member_constant);

            let visibility_tag = match visibility {
                Visibility::Protected => 1_i64,
                Visibility::Private => 2_i64,
                Visibility::Public => unreachable!("un membre public ne peut pas être restreint"),
            };
            let tag_constant = self.make_constant(Value::Integer(visibility_tag))?;
            self.emit_constant_op(OpCode::Constant, tag_constant);
        }

        self.emit_byte(OpCode::Class.into());
        self.emit_byte(runtime_bases.len() as u8);
        self.emit_byte(instance_methods.len() as u8);
        self.emit_byte(static_methods.len() as u8);
        self.emit_byte(static_fields.len() as u8);
        self.emit_byte(restricted_members.len() as u8);

        if !self.in_function && self.scope_depth == 0 {
            // Réutilise la constante déjà enregistrée par la pré-déclaration
            // plutôt que d'en créer (et d'insérer) une nouvelle entrée.
            let name_constant = if self.predeclared_functions.contains(name) {
                self.globals
                    .borrow()
                    .get(name)
                    .map(|global| global.constant)
                    .ok_or_else(|| CompileError::VariableAlreadyDeclared(name.to_string()))?
            } else {
                let constant = self.identifier_constant(name)?;

                self.globals.borrow_mut().insert(
                    name.to_string(),
                    Global {
                        constant,
                        mutable: true,
                        native: false,
                        is_function: false,
                    },
                );

                constant
            };

            self.emit_constant_op(OpCode::DefineGlobal, name_constant);
        } else {
            let slot =
                self.context
                    .borrow_mut()
                    .locals
                    .declare_local(name, self.scope_depth, true)?;

            self.context
                .borrow_mut()
                .locals
                .mark_initialized(self.scope_depth);

            debug_assert_eq!(self.context.borrow().locals.len() - 1, slot as usize);
        }

        Ok(())
    }

    // ============================================================
    // ENUM
    // ============================================================

    fn type_expr_name(type_expr: &TypeExpr) -> Option<String> {
        match type_expr {
            TypeExpr::Named(name) => Some(name.clone()),
            TypeExpr::Generic { name, .. } => Some(name.clone()),
            _ => None,
        }
    }

    pub(crate) fn compile_enum(
        &mut self,
        name: &str,
        variants: &[String],
        methods: &[FunctionMethod],
    ) -> Result<(), CompileError> {
        if variants.len() > u8::MAX as usize || methods.len() > u8::MAX as usize {
            return Err(CompileError::TooManyObjectFields);
        }

        if !self.in_function
            && self.scope_depth == 0
            && !self.predeclared_functions.contains(name)
            && let Some(global) = self.globals.borrow().get(name)
            && !global.native
        {
            return Err(CompileError::VariableAlreadyDeclared(name.to_string()));
        }

        let name_constant = self.identifier_constant(name)?;
        self.emit_constant_op(OpCode::Constant, name_constant);

        for variant in variants {
            let variant_constant = self.identifier_constant(variant)?;
            self.emit_constant_op(OpCode::Constant, variant_constant);
        }

        for method in methods {
            let method_name_constant = self.identifier_constant(&method.name)?;
            self.emit_constant_op(OpCode::Constant, method_name_constant);

            let function = self.compile_method(&method.name, &method.params, &method.body)?;
            let function_constant =
                self.make_constant(Value::new_function(std::rc::Rc::new(function.clone())))?;

            self.emit_closure(function_constant, &function.upvalues);
        }

        self.emit_byte(OpCode::Enum.into());
        self.emit_byte(variants.len() as u8);
        self.emit_byte(methods.len() as u8);

        if !self.in_function && self.scope_depth == 0 {
            let name_constant = if self.predeclared_functions.contains(name) {
                self.globals
                    .borrow()
                    .get(name)
                    .map(|global| global.constant)
                    .ok_or_else(|| CompileError::VariableAlreadyDeclared(name.to_string()))?
            } else {
                let constant = self.identifier_constant(name)?;

                self.globals.borrow_mut().insert(
                    name.to_string(),
                    Global {
                        constant,
                        mutable: true,
                        native: false,
                        is_function: false,
                    },
                );

                constant
            };

            self.emit_constant_op(OpCode::DefineGlobal, name_constant);
        } else {
            let slot =
                self.context
                    .borrow_mut()
                    .locals
                    .declare_local(name, self.scope_depth, true)?;

            self.context
                .borrow_mut()
                .locals
                .mark_initialized(self.scope_depth);

            debug_assert_eq!(self.context.borrow().locals.len() - 1, slot as usize);
        }

        Ok(())
    }

    pub(crate) fn compile_interface(
        &mut self,
        name: &str,
        bases: &[TypeExpr],
        methods: &[InterfaceMethod],
    ) -> Result<(), CompileError> {
        if bases.len() > u8::MAX as usize {
            return Err(CompileError::TooManyObjectFields);
        }

        if methods.len() > u8::MAX as usize {
            return Err(CompileError::TooManyObjectFields);
        }

        if !self.in_function
            && self.scope_depth == 0
            && !self.predeclared_functions.contains(name)
            && let Some(global) = self.globals.borrow().get(name)
            && !global.native
        {
            return Err(CompileError::VariableAlreadyDeclared(name.to_string()));
        }

        let runtime_bases: Vec<&TypeExpr> = bases
            .iter()
            .filter(|base| {
                let Some(name) = Self::type_expr_name(base) else {
                    return false;
                };

                crate::compiler::capability::Capability::from_name(&name).is_none()
                    && !matches!(name.as_str(), "Iterator" | "Iterable")
            })
            .collect();

        for base in &runtime_bases {
            let base_name = Self::type_expr_name(base).ok_or_else(|| {
                CompileError::InternalCompilerError(
                    "Une base d'interface générique doit désigner un type nommé".to_string(),
                )
            })?;
            self.compile_variable_get(&base_name)?;
        }

        let name_constant = self.identifier_constant(name)?;

        self.emit_constant_op(OpCode::Constant, name_constant);

        for method in methods {
            let method_constant = self.identifier_constant(&method.name)?;

            self.emit_constant_op(OpCode::Constant, method_constant);

            let arity_constant = self.make_constant(Value::Integer(method.arity as i64))?;

            self.emit_constant_op(OpCode::Constant, arity_constant);
        }

        self.emit_byte(OpCode::Interface.into());
        self.emit_byte(runtime_bases.len() as u8);
        self.emit_byte(methods.len() as u8);

        if !self.in_function && self.scope_depth == 0 {
            let name_constant = if self.predeclared_functions.contains(name) {
                self.globals
                    .borrow()
                    .get(name)
                    .map(|global| global.constant)
                    .ok_or_else(|| CompileError::VariableAlreadyDeclared(name.to_string()))?
            } else {
                let constant = self.identifier_constant(name)?;

                self.globals.borrow_mut().insert(
                    name.to_string(),
                    Global {
                        constant,
                        mutable: true,
                        native: false,
                        is_function: false,
                    },
                );

                constant
            };

            self.emit_constant_op(OpCode::DefineGlobal, name_constant);
        } else {
            let slot =
                self.context
                    .borrow_mut()
                    .locals
                    .declare_local(name, self.scope_depth, true)?;

            self.context
                .borrow_mut()
                .locals
                .mark_initialized(self.scope_depth);

            debug_assert_eq!(self.context.borrow().locals.len() - 1, slot as usize);
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
        catch_type: Option<&TypeExpr>,
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

        let has_finally = finally_body.is_some();

        self.emit_opcode(OpCode::PushExceptionHandler);

        let catch_operand = self.chunk.code.len();
        self.emit_u16(u16::MAX);

        let finally_operand = self.chunk.code.len();
        self.emit_u16(u16::MAX);

        let catch_type_constant = match catch_type {
            None => u16::MAX,
            Some(TypeExpr::Named(name)) => self.identifier_constant(name)?,
            Some(_) => {
                return Err(CompileError::InternalCompilerError(
                    "Le type d'un catch doit être un type nommé, par exemple `Err`.".to_string(),
                ));
            }
        };
        self.emit_u16(catch_type_constant);

        // Le `try` est « ouvert » pendant son corps et son `catch` : un
        // `return`/`break`/`continue` qui en sort dépile le handler et inline
        // le `finally`.
        self.push_try_context(finally_body);

        self.begin_scope();

        for statement in try_body {
            self.compile_statement(statement)?;
        }

        self.emit_opcode(OpCode::PopExceptionHandler);

        self.end_scope();

        // Fin normale du `try` : le `finally` reçoit l'état `[None, false]`
        // (pas d'exception en attente). Le `finally` atteint par exception
        // reçoit `[valeur, true]`, empilé par la VM.
        if has_finally {
            self.emit_opcode(OpCode::None);
            self.emit_opcode(OpCode::False);
        }

        let normal_end_jump = self.emit_jump(OpCode::Jump);

        let mut catch_end_jump = None;

        let catch_ip;

        if let Some(body) = catch_body {
            catch_ip = self.chunk.code.len();

            // Dans le `catch`, la VM a retiré le handler s'il n'y a pas de
            // `finally` ; sinon il reste actif pour garder ce `finally`.
            self.set_try_handler_active(has_finally);

            self.begin_scope();

            // La VM empile TOUJOURS la valeur interceptée : elle doit avoir une
            // locale, même quand le `catch` n'est pas nommé.
            self.declare_existing_local(catch_name.unwrap_or("__catch_value"), true)?;

            for statement in body {
                self.compile_statement(statement)?;
            }

            self.end_scope();

            if has_finally {
                self.emit_opcode(OpCode::PopExceptionHandler);
                self.emit_opcode(OpCode::None);
                self.emit_opcode(OpCode::False);
            }

            let jump = self.emit_jump(OpCode::Jump);
            catch_end_jump = Some(jump);
        } else {
            catch_ip = self.chunk.code.len();
        }

        // Le corps du `finally` est compilé hors du contexte de son `try`.
        self.pop_try_context();

        let finally_ip = if let Some(body) = finally_body {
            let ip = self.chunk.code.len();

            // Deux locales cachées : l'état `[valeur, drapeau]` du `finally`.
            self.begin_scope();
            self.declare_existing_local("__finally_value", true)?;
            self.declare_existing_local("__finally_flag", true)?;

            self.begin_scope();

            for statement in body {
                self.compile_statement(statement)?;
            }

            self.end_scope();

            // `FinallyEnd` consomme lui-même les deux valeurs : aucun `Pop`.
            self.emit_opcode(OpCode::FinallyEnd);
            self.discard_scope();

            Some(ip)
        } else {
            None
        };

        // `u16::MAX` code « pas de catch / pas de finally » dans l'opérande :
        // une adresse réelle égale à cette valeur serait lue comme absente
        // par la VM (le handler perdrait silencieusement son catch/finally).
        if catch_body.is_some() && catch_ip >= u16::MAX as usize {
            return Err(CompileError::JumpTooLarge);
        }

        if finally_ip.is_some_and(|ip| ip >= u16::MAX as usize) {
            return Err(CompileError::JumpTooLarge);
        }

        if catch_body.is_some() {
            self.patch_u16(catch_operand, catch_ip)?;
        } else {
            self.patch_u16(catch_operand, u16::MAX as usize)?;
        }

        if let Some(ip) = finally_ip {
            self.patch_u16(finally_operand, ip)?;
        } else {
            self.patch_u16(finally_operand, u16::MAX as usize)?;
        }

        if let Some(jump) = catch_end_jump {
            let target = finally_ip.unwrap_or(self.chunk.code.len());
            self.patch_jump_to(jump, target)?;
        }

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

            // Les bindings d'un pattern sont réservés UNE FOIS au début de
            // l'arm. Le code de test du pattern peut ensuite les affecter
            // selon l'alternative effectivement prise (`x | y`, `Some(x) |
            // Ok(x)`, patterns imbriqués, ...), sans redéclaration locale.
            self.declare_pattern_bindings(&arm.pattern)?;

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
        self.compile_pattern_test_expression(&subject, pattern)
    }

    fn declare_pattern_bindings(&mut self, pattern: &Pattern) -> Result<(), CompileError> {
        let mut names = Vec::new();
        Self::collect_pattern_binding_names(pattern, &mut names);

        for name in names {
            // `None` n'est qu'une valeur temporaire dans le slot. Le pattern
            // l'écrasera avant que le guard ou le corps de l'arm ne puisse
            // lire la variable.
            self.compile_local_var(&name, None, true)?;
        }

        Ok(())
    }

    fn collect_pattern_binding_names(pattern: &Pattern, names: &mut Vec<String>) {
        let mut push_name = |name: &str| {
            if !names.iter().any(|existing| existing == name) {
                names.push(name.to_string());
            }
        };

        match pattern {
            Pattern::Binding(name) => push_name(name),

            Pattern::Wildcard | Pattern::Literal(_) | Pattern::EnumVariant { .. } => {}

            Pattern::Or(patterns) => {
                for pattern in patterns {
                    Self::collect_pattern_binding_names(pattern, names);
                }
            }

            Pattern::Range { start, end, .. } => {
                Self::collect_pattern_binding_names(start, names);
                Self::collect_pattern_binding_names(end, names);
            }

            Pattern::Array(patterns) | Pattern::ArrayRest(patterns) | Pattern::Tuple(patterns) => {
                for pattern in patterns {
                    Self::collect_pattern_binding_names(pattern, names);
                }
            }

            Pattern::OptionSome(pattern)
            | Pattern::ResultOk(pattern)
            | Pattern::ResultErr(pattern) => {
                Self::collect_pattern_binding_names(pattern, names);
            }
        }
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

            Pattern::Binding(name) => {
                // Les bindings sont déjà déclarés par `declare_pattern_bindings`.
                self.compile_expression(expression)?;
                self.compile_variable_set(name)?;
                self.emit_opcode(OpCode::Pop);
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

            Pattern::Array(patterns) => {
                self.compile_sequence_pattern_expression(expression, patterns, false)
            }

            Pattern::ArrayRest(patterns) => {
                self.compile_sequence_pattern_expression(expression, patterns, true)
            }

            Pattern::Tuple(patterns) => self.compile_tuple_pattern_expression(expression, patterns),

            Pattern::OptionSome(pattern) => self
                .compile_option_result_pattern(expression, "Option", "is_some", "unwrap", pattern),

            Pattern::ResultOk(pattern) => {
                self.compile_option_result_pattern(expression, "Result", "is_ok", "unwrap", pattern)
            }

            Pattern::ResultErr(pattern) => self.compile_option_result_pattern(
                expression,
                "Result",
                "is_err",
                "unwrap_err",
                pattern,
            ),

            Pattern::EnumVariant {
                enum_name,
                variant_name,
            } => {
                let variant = Expression::Member {
                    object: Box::new(Expression::Variable(enum_name.clone())),
                    name: variant_name.clone(),
                    line: 0,
                    column: 0,
                };

                self.compile_expression(expression)?;
                self.compile_expression(&variant)?;
                self.emit_opcode(OpCode::Equal);
                Ok(self.emit_jump(OpCode::JumpIfFalse))
            }
        }
    }

    fn compile_option_result_pattern(
        &mut self,
        expression: &Expression,
        type_name: &str,
        discriminator: &str,
        extractor: &str,
        inner: &Pattern,
    ) -> Result<usize, CompileError> {
        let type_guard = self.type_guard_expression(expression, type_name);
        self.compile_expression(&type_guard)?;
        let type_false_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_opcode(OpCode::Pop);

        self.compile_method_call(expression, discriminator, &[], 0, 0)?;
        let discriminator_false_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_opcode(OpCode::Pop);

        let extracted = self.pattern_method_expression(expression, extractor);
        let inner_false_jump = self.compile_pattern_test_expression(&extracted, inner)?;

        self.patch_jump(type_false_jump)?;
        self.patch_jump(discriminator_false_jump)?;

        // `inner_false_jump` doit également converger vers le même booléen
        // final. Si le sous-pattern réussit, sa valeur `true` tombe ici sans
        // être modifiée.
        self.patch_jump(inner_false_jump)?;

        Ok(self.emit_jump(OpCode::JumpIfFalse))
    }

    fn pattern_method_expression(&self, object: &Expression, name: &str) -> Expression {
        Expression::Call {
            callee: Box::new(Expression::Member {
                object: Box::new(object.clone()),
                name: name.to_string(),
                line: 0,
                column: 0,
            }),
            generic_args: Vec::new(),
            arguments: Vec::new(),
            line: 0,
            column: 0,
        }
    }

    fn type_guard_expression(&self, expression: &Expression, expected: &str) -> Expression {
        Expression::Binary {
            left: Box::new(Expression::Call {
                callee: Box::new(Expression::Variable("type".to_string())),
                generic_args: Vec::new(),
                arguments: vec![expression.clone()],
                line: 0,
                column: 0,
            }),
            operator: BinaryOp::Equal,
            right: Box::new(Expression::Literal(Literal::String(expected.to_string()))),
            line: 0,
            column: 0,
        }
    }

    // ============================================================
    // LITERAL
    // ============================================================

    fn compile_literal_pattern(&mut self, literal: &Literal) -> Result<(), CompileError> {
        match literal {
            Literal::Integer(value) => {
                let constant = self.make_constant(Value::Integer(*value))?;
                self.emit_constant_op(OpCode::Constant, constant);
            }

            Literal::Float(value) => {
                let constant = self.make_constant(Value::Float(*value))?;
                self.emit_constant_op(OpCode::Constant, constant);
            }

            Literal::String(value) => {
                let constant = self.make_constant(Value::new_string(value.clone()))?;
                self.emit_constant_op(OpCode::Constant, constant);
            }

            Literal::Bool(true) => self.emit_opcode(OpCode::True),
            Literal::Bool(false) => self.emit_opcode(OpCode::False),
            Literal::None => self.emit_opcode(OpCode::None),
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

        // Les branches qui ont réussi arrivent ici avec `true`; la dernière
        // alternative utilise son propre booléen et son propre saut d'échec.
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
        // Un pattern de range doit rester sûr avec une expression dynamique :
        // on refuse ainsi de transformer un simple non-match en TypeError.
        let numeric_guard = Expression::Binary {
            left: Box::new(self.type_guard_expression(expression, "int")),
            operator: BinaryOp::Or,
            right: Box::new(self.type_guard_expression(expression, "float")),
            line: 0,
            column: 0,
        };

        self.compile_expression(&numeric_guard)?;
        let type_false_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_opcode(OpCode::Pop);

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

        // Les chemins négatifs ont déjà laissé `false` sur la pile.
        // Ils convergent vers un seul `JumpIfFalse`; le chemin succès garde
        // `true` et saute ce test final.
        let success_jump = self.emit_jump(OpCode::Jump);

        self.patch_jump(type_false_jump)?;
        self.patch_jump(lower_false_jump)?;
        self.patch_jump(upper_false_jump)?;

        let final_false_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.patch_jump(success_jump)?;

        Ok(final_false_jump)
    }

    // ============================================================
    // ARRAY / TUPLE
    // ============================================================

    fn compile_sequence_pattern_expression(
        &mut self,
        expression: &Expression,
        patterns: &[Pattern],
        rest: bool,
    ) -> Result<usize, CompileError> {
        let type_guard = self.type_guard_expression(expression, "list");
        self.compile_expression(&type_guard)?;
        let type_false_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_opcode(OpCode::Pop);

        if rest && patterns.is_empty() {
            // `[..]` accepte toute liste : après le guard de type, on n'a
            // aucun élément à vérifier et la longueur ne nous intéresse pas.
            self.compile_expression(expression)?;
            self.emit_opcode(OpCode::ArrayLength);
            self.emit_opcode(OpCode::Pop);
            self.emit_opcode(OpCode::True);

            let success_jump = self.emit_jump(OpCode::Jump);
            self.patch_jump(type_false_jump)?;
            self.emit_opcode(OpCode::Pop);
            self.emit_opcode(OpCode::False);
            self.patch_jump(success_jump)?;

            return Ok(self.emit_jump(OpCode::JumpIfFalse));
        }

        self.compile_expression(expression)?;
        self.emit_opcode(OpCode::ArrayLength);

        if rest {
            // length >= N  <=>  length > N - 1
            let lower = self.make_constant(Value::Integer((patterns.len() - 1) as i64))?;
            self.emit_constant_op(OpCode::Constant, lower);
            self.emit_opcode(OpCode::Greater);
        } else {
            let count_constant = self.make_constant(Value::Integer(patterns.len() as i64))?;
            self.emit_constant_op(OpCode::Constant, count_constant);
            self.emit_opcode(OpCode::Equal);
        }

        let length_false_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_opcode(OpCode::Pop);

        let mut element_false_jumps = Vec::with_capacity(patterns.len());

        for (index, pattern) in patterns.iter().enumerate() {
            let element_expression = Expression::Index {
                object: Box::new(expression.clone()),
                index: Box::new(Expression::Literal(Literal::Integer(index as i64))),
                line: 0,
                column: 0,
            };

            let false_jump = self.compile_pattern_test_expression(&element_expression, pattern)?;
            self.emit_opcode(OpCode::Pop);
            element_false_jumps.push(false_jump);
        }

        self.emit_opcode(OpCode::True);
        let success_jump = self.emit_jump(OpCode::Jump);

        self.patch_jump(type_false_jump)?;
        self.emit_opcode(OpCode::Pop);
        self.emit_opcode(OpCode::False);
        let mut result_jumps = vec![self.emit_jump(OpCode::Jump)];

        self.patch_jump(length_false_jump)?;
        self.emit_opcode(OpCode::Pop);
        self.emit_opcode(OpCode::False);
        result_jumps.push(self.emit_jump(OpCode::Jump));

        for false_jump in element_false_jumps {
            self.patch_jump(false_jump)?;
            self.emit_opcode(OpCode::Pop);
            self.emit_opcode(OpCode::False);
            result_jumps.push(self.emit_jump(OpCode::Jump));
        }

        self.patch_jump(success_jump)?;

        for jump in result_jumps {
            self.patch_jump(jump)?;
        }

        Ok(self.emit_jump(OpCode::JumpIfFalse))
    }

    fn compile_tuple_pattern_expression(
        &mut self,
        expression: &Expression,
        patterns: &[Pattern],
    ) -> Result<usize, CompileError> {
        let type_guard = self.type_guard_expression(expression, "tuple");
        self.compile_expression(&type_guard)?;
        let type_false_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_opcode(OpCode::Pop);

        self.compile_expression(expression)?;
        self.emit_opcode(OpCode::ArrayLength);
        let length_constant = self.make_constant(Value::Integer(patterns.len() as i64))?;
        self.emit_constant_op(OpCode::Constant, length_constant);
        self.emit_opcode(OpCode::Equal);
        let length_false_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_opcode(OpCode::Pop);

        let mut element_false_jumps = Vec::with_capacity(patterns.len());

        for (index, pattern) in patterns.iter().enumerate() {
            let element_expression = Expression::Index {
                object: Box::new(expression.clone()),
                index: Box::new(Expression::Literal(Literal::Integer(index as i64))),
                line: 0,
                column: 0,
            };

            let false_jump = self.compile_pattern_test_expression(&element_expression, pattern)?;
            self.emit_opcode(OpCode::Pop);
            element_false_jumps.push(false_jump);
        }

        self.emit_opcode(OpCode::True);
        let success_jump = self.emit_jump(OpCode::Jump);

        let mut result_jumps = Vec::new();

        self.patch_jump(type_false_jump)?;
        self.emit_opcode(OpCode::Pop);
        self.emit_opcode(OpCode::False);
        result_jumps.push(self.emit_jump(OpCode::Jump));

        self.patch_jump(length_false_jump)?;
        self.emit_opcode(OpCode::Pop);
        self.emit_opcode(OpCode::False);
        result_jumps.push(self.emit_jump(OpCode::Jump));

        for false_jump in element_false_jumps {
            self.patch_jump(false_jump)?;
            self.emit_opcode(OpCode::Pop);
            self.emit_opcode(OpCode::False);
            result_jumps.push(self.emit_jump(OpCode::Jump));
        }

        self.patch_jump(success_jump)?;

        for jump in result_jumps {
            self.patch_jump(jump)?;
        }

        Ok(self.emit_jump(OpCode::JumpIfFalse))
    }

    // ============================================================
    // HELPERS
    // ============================================================

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

        // ------------------------------------------------------------
        // from dog import *
        // ------------------------------------------------------------

        if items.len() == 1 && items[0].name == "*" {
            if items[0].alias.is_some() {
                return Err(CompileError::InvalidImport);
            }

            let module_constant = self.make_constant(Value::new_string(module_name.clone()))?;

            self.emit_constant_op(OpCode::ImportAll, module_constant);

            self.imported_modules.insert(module_name);
            self.wildcard_imported = true;

            return Ok(());
        }

        // ------------------------------------------------------------
        // from dog import Dog, Animal, details
        //
        // import dog {
        //     Dog,
        //     Animal,
        //     details
        // }
        // ------------------------------------------------------------

        for item in items {
            if item.name == "*" {
                return Err(CompileError::InvalidImport);
            }

            // `from m import Person;` où `Person` n'est qu'un alias de
            // type : rien à faire à l'exécution.
            let full_path: Vec<String> = module
                .parts
                .iter()
                .cloned()
                .chain(std::iter::once(item.name.clone()))
                .collect();

            if self.is_type_only_import(&full_path) {
                continue;
            }

            let binding_name = item.alias.as_deref().unwrap_or(&item.name);

            if let Some(global) = self.globals.borrow().get(binding_name)
                && !global.native
            {
                return Err(CompileError::VariableAlreadyDeclared(
                    binding_name.to_string(),
                ));
            }

            let module_constant = self.make_constant(Value::new_string(module_name.clone()))?;

            self.emit_constant_op(OpCode::Import, module_constant);

            let property_constant = self.identifier_constant(&item.name)?;

            self.emit_constant_op(OpCode::GetProperty, property_constant);

            let binding_constant = self.identifier_constant(binding_name)?;

            self.emit_constant_op(OpCode::DefineGlobal, binding_constant);

            self.globals.borrow_mut().insert(
                binding_name.to_string(),
                Global {
                    constant: binding_constant,
                    mutable: false,
                    native: false,
                    is_function: false,
                },
            );
        }

        self.imported_modules.insert(module_name);

        Ok(())
    }

    pub(crate) fn compile_import(&mut self, path: &[String]) -> Result<(), CompileError> {
        if path.is_empty() {
            return Err(CompileError::InvalidImport);
        }

        // ------------------------------------------------------------
        // import module;
        // import module.sub;
        // import module.Export;
        //
        // La distinction entre "sous-module" (import std.math -> la
        // variable `math` est le MODULE std/math.ks, ce qui permet
        // ensuite `math.sqrt(2.0)`) et "export tiré d'un module"
        // (import shapes.Circle -> `Circle` est directement la
        // CLASSE exportée par shapes.ks, ce qui permet `new Circle()`
        // sans qualifier par le nom du module) n'est PAS tranchée ici,
        // à la compilation : elle dépend de l'existence réelle d'un
        // fichier std/math.ks sur le disque, information que seule la
        // VM possède au moment de charger le module. C'est donc
        // `import_module`, côté VM, qui essaie d'abord le chemin
        // complet comme sous-module, puis se replie sur "tout sauf le
        // dernier segment = module, dernier segment = export" si le
        // chemin complet n'existe pas (voir
        // `VirtualMachine::resolve_import_value` dans
        // vm/machine/modules.rs). Le compilateur, lui, se contente
        // toujours de lier le DERNIER segment du chemin — c'est cette
        // même règle dans les deux cas (module ou export) qui rend
        // possible de laisser la VM décider.
        //
        // Exemples :
        //     import dog;             // dog     = module dog.ks
        //     import std.math;        // math    = module std/math.ks
        //     import shapes.Circle;   // Circle  = export Circle de shapes.ks
        // ------------------------------------------------------------

        let module_name = path.join(".");

        let binding_name = path
            .last()
            .map(String::as_str)
            .ok_or(CompileError::InvalidImport)?;

        if binding_name.is_empty() {
            return Err(CompileError::InvalidImport);
        }

        // `import m.Person;` où `Person` n'est qu'un alias de type : rien à
        // faire à l'exécution (le vérificateur de types l'a déjà validé).
        if self.is_type_only_import(path) {
            return Ok(());
        }

        if let Some(global) = self.globals.borrow().get(binding_name)
            && !global.native
        {
            return Err(CompileError::VariableAlreadyDeclared(
                binding_name.to_string(),
            ));
        }

        // ------------------------------------------------------------
        // Charger le module (ou l'export qu'il contient — voir plus
        // haut) et lier le résultat au dernier segment du chemin.
        // ------------------------------------------------------------

        let module_constant = self.make_constant(Value::new_string(module_name.clone()))?;

        self.emit_constant_op(OpCode::Import, module_constant);

        let binding_constant = self.identifier_constant(binding_name)?;

        self.emit_constant_op(OpCode::DefineGlobal, binding_constant);

        self.globals.borrow_mut().insert(
            binding_name.to_string(),
            Global {
                constant: binding_constant,
                mutable: false,
                native: false,
                is_function: false,
            },
        );

        // Évite de recharger le même module.
        self.imported_modules.insert(module_name);

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
                // Plusieurs `export func` de même nom = surcharges d'UN seul
                // export (les doublons de même arité sont refusés à la
                // prédéclaration).
                if !self.exports.iter().any(|export| export == name) {
                    self.register_export(name)?;
                }

                self.compile_statement(statement)?;
            }

            Statement::Class { name, .. } => {
                self.register_export(name)?;
                self.compile_statement(statement)?;
            }

            Statement::Interface { name, .. } => {
                self.register_export(name)?;
                self.compile_statement(statement)?;
            }

            Statement::Enum { name, .. } => {
                self.register_export(name)?;
                self.compile_statement(statement)?;
            }

            // Un alias de type n'a aucune existence à l'exécution (pas de
            // global défini) : il n'entre donc PAS dans la liste des exports
            // runtime du module. Son export passe uniquement par
            // l'interface de TYPES du module (voir `ModuleTypeInterface`,
            // qui contient les alias exportés à part).
            Statement::TypeAlias { .. } => {}

            _ => {
                return Err(CompileError::InvalidExport);
            }
        }

        Ok(())
    }

    /// REPL sans contexte de résolution (les imports y sont refusés).
    #[allow(dead_code)]
    pub(crate) fn compile_repl(self, statements: &[Statement]) -> Result<Function, CompileError> {
        self.compile_repl_inner(statements, None)
    }

    /// REPL avec contexte de résolution : `import` et `from ... import`
    /// fonctionnent (les chemins sont résolus à partir du répertoire courant).
    pub(crate) fn compile_repl_with_context(
        self,
        statements: &[Statement],
        context: TypeCheckContext,
    ) -> Result<Function, CompileError> {
        self.compile_repl_inner(statements, Some(context))
    }

    fn compile_repl_inner(
        mut self,
        statements: &[Statement],
        context: Option<TypeCheckContext>,
    ) -> Result<Function, CompileError> {
        match context.clone() {
            Some(context) => TypeChecker::check_with_context(statements, context)?,
            None => TypeChecker::check(statements)?,
        }

        self.type_context = context;

        for (index, statement) in statements.iter().enumerate() {
            let is_last = index + 1 == statements.len();

            match (is_last, statement) {
                (
                    true,
                    Statement::Positioned {
                        line,
                        column,
                        statement,
                    },
                ) if matches!(statement.as_ref(), Statement::Expression { .. }) => {
                    self.current_line = *line;
                    self.current_column = *column;

                    if let Statement::Expression { expression } = statement.as_ref() {
                        self.compile_expression(expression)?;
                    }
                }

                (true, Statement::Expression { expression }) => {
                    self.compile_expression(expression)?;
                }

                _ => {
                    self.compile_statement(statement)?;
                }
            }
        }

        self.emit_opcode(OpCode::Halt);

        let local_count = u8::try_from(self.context.borrow().locals.max_slots())
            .map_err(|_| CompileError::TooManyLocals)?;

        Ok(Function {
            name: "<repl>".to_string(),
            arity: 0,
            is_async: false,
            chunk: std::rc::Rc::new(self.chunk),
            local_count: local_count.into(),
            upvalue_count: 0,
            upvalues: Vec::new(),
        })
    }
}

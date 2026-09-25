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
                name, params, body, ..
            } => {
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
        if !self.in_function && self.scope_depth == 0 && !self.predeclared_functions.contains(name)
        {
            if let Some(global) = self.globals.borrow().get(name) {
                if !global.native {
                    return Err(CompileError::VariableAlreadyDeclared(name.to_string()));
                }
            }
        }

        for base in bases {
            let base_name = Self::type_expr_name(base).ok_or_else(|| {
                CompileError::InternalCompilerError(
                    "Une base d'héritage générique doit désigner un type nommé".to_string(),
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
        self.emit_byte(bases.len() as u8);
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

        if !self.in_function && self.scope_depth == 0 && !self.predeclared_functions.contains(name) {
            if let Some(global) = self.globals.borrow().get(name) {
                if !global.native {
                    return Err(CompileError::VariableAlreadyDeclared(name.to_string()));
                }
            }
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
            let slot = self
                .context
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

        if !self.in_function && self.scope_depth == 0 && !self.predeclared_functions.contains(name)
        {
            if let Some(global) = self.globals.borrow().get(name) {
                if !global.native {
                    return Err(CompileError::VariableAlreadyDeclared(name.to_string()));
                }
            }
        }

        for base in bases {
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
        self.emit_byte(bases.len() as u8);
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

        self.emit_opcode(OpCode::PushExceptionHandler);

        let catch_operand = self.chunk.code.len();
        self.emit_u16(u16::MAX);

        let finally_operand = self.chunk.code.len();
        self.emit_u16(u16::MAX);

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

        // `test_false_jump` doit être fusionné (patché) AVANT que l'on émette
        // le `JumpIfFalse` final : sinon, le chemin "échec" saute par-dessus
        // ce dernier (il atterrit après lui) et ce jump ne lit alors jamais la
        // valeur `false` laissée par le test — il ne se déclenche donc jamais,
        // quel que soit le pattern, et l'exécution retombe systématiquement
        // dans le corps du bras courant (voir compile_range_pattern_expression
        // / compile_array_pattern_expression pour l'ordre correct : patch
        // d'abord, puis émission du JumpIfFalse qui lit la valeur fusionnée).
        self.patch_jump(test_false_jump)?;

        let final_false_jump = self.emit_jump(OpCode::JumpIfFalse);

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

            Literal::Bool(true) => {
                self.emit_opcode(OpCode::True);
            }

            Literal::Bool(false) => {
                self.emit_opcode(OpCode::False);
            }

            Literal::None => {
                self.emit_opcode(OpCode::None);
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

        self.emit_constant_op(OpCode::Constant, length_constant);

        self.emit_opcode(OpCode::Equal);

        let length_false_jump = self.emit_jump(OpCode::JumpIfFalse);

        self.emit_opcode(OpCode::Pop);

        let mut element_false_jumps = Vec::new();

        for (index, pattern) in patterns.iter().enumerate() {
            let element_expression = Expression::Index {
                object: Box::new(expression.clone()),
                index: Box::new(Expression::Literal(Literal::Integer(index as i64))),
                // Expression synthétisée par le compilateur (pas de position
                // source réelle) : aucune valeur pertinente à fournir ici.
                line: 0,
                column: 0,
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
                        line: 0,
                        column: 0,
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

            if let Some(global) = self.globals.borrow().get(binding_name) {
                if !global.native {
                    return Err(CompileError::VariableAlreadyDeclared(
                        binding_name.to_string(),
                    ));
                }
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

        if let Some(global) = self.globals.borrow().get(binding_name) {
            if !global.native {
                return Err(CompileError::VariableAlreadyDeclared(
                    binding_name.to_string(),
                ));
            }
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
            chunk: std::rc::Rc::new(self.chunk),
            local_count: local_count.into(),
            upvalue_count: 0,
            upvalues: Vec::new(),
        })
    }
}

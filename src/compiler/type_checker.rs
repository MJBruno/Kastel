use std::collections::{HashMap, HashSet};

use crate::error::compile_error::CompileError;
use crate::frontend::ast::*;

use super::{
    builtin_types,
    module_types::{ImportedType, ModuleTypeLoader},
    types::{FunctionType, Type},
};

use std::{path::PathBuf, rc::Rc};

#[derive(Clone)]
pub struct TypeCheckContext {
    pub current_module: PathBuf,
    pub module_loader: Rc<ModuleTypeLoader>,
}

impl TypeCheckContext {
    pub fn new(current_module: PathBuf, module_loader: Rc<ModuleTypeLoader>) -> Self {
        Self {
            current_module,
            module_loader,
        }
    }
}

#[derive(Debug, Clone)]
struct Binding {
    ty: Type,
    _mutable: bool,
    native: bool,
}

#[derive(Debug, Clone)]
struct ClassInfo {
    bases: Vec<String>,
    methods: HashMap<String, FunctionType>,
}

/// Vérificateur statique graduel de Kastel.
///
/// Règle centrale : `Dynamic` ne bloque jamais un programme. Une erreur
/// n'est produite que lorsqu'une incompatibilité est certaine à la compilation.
pub struct TypeChecker {
    scopes: Vec<HashMap<String, Binding>>,
    functions: HashMap<String, FunctionType>,
    classes: HashMap<String, ClassInfo>,
    parents: HashMap<String, Vec<String>>,
    current_return_type: Option<Type>,
    return_types: Vec<Type>,
    current_class: Option<String>,
    context: Option<TypeCheckContext>,
}

impl TypeChecker {
    pub fn check(statements: &[Statement]) -> Result<(), CompileError> {
        let mut checker = Self::new();
        checker.collect_declarations(statements)?;
        checker.check_statements(statements)
    }

    pub fn check_with_context(
        statements: &[Statement],
        context: TypeCheckContext,
    ) -> Result<(), CompileError> {
        let mut checker = Self::new_with_context(context);
        checker.collect_declarations(statements)?;
        checker.check_statements(statements)
    }

    pub fn analyze_module(
        statements: &[Statement],
        context: TypeCheckContext,
    ) -> Result<HashMap<String, Type>, CompileError> {
        let mut checker = Self::new_with_context(context);
        checker.collect_declarations(statements)?;
        checker.check_statements(statements)?;

        let mut exports = HashMap::new();
        for statement in statements {
            let Statement::Export { statement } = Self::strip_position(statement) else {
                continue;
            };
            let (name, ty) = checker.export_type(Self::strip_position(statement))?;
            if exports.insert(name.clone(), ty).is_some() {
                return Err(CompileError::DuplicateExport(name));
            }
        }

        Ok(exports)
    }

    fn new() -> Self {
        let mut globals = HashMap::new();
        for (name, ty) in builtin_types::all() {
            globals.insert(
                name,
                Binding {
                    ty,
                    _mutable: false,
                    native: true,
                },
            );
        }

        Self {
            scopes: vec![globals],
            functions: HashMap::new(),
            classes: HashMap::new(),
            parents: HashMap::new(),
            current_return_type: None,
            return_types: Vec::new(),
            current_class: None,
            context: None,
        }
    }

    fn new_with_context(context: TypeCheckContext) -> Self {
        let mut checker = Self::new();
        checker.context = Some(context);
        checker
    }

    fn collect_declarations(&mut self, statements: &[Statement]) -> Result<(), CompileError> {
        for statement in statements {
            match statement {
                Statement::Positioned { statement, .. } => {
                    self.collect_declarations(std::slice::from_ref(statement))?;
                }

                Statement::Export { statement } => {
                    self.collect_declarations(std::slice::from_ref(statement))?;
                }

                Statement::Function {
                    name,
                    params,
                    param_types,
                    return_type,
                    ..
                } => {
                    let parameters = params
                        .iter()
                        .enumerate()
                        .map(|(index, _)| {
                            param_types
                                .get(index)
                                .and_then(|annotation| annotation.as_ref())
                                .map_or(Type::Dynamic, |annotation| {
                                    Type::from_type_expr(annotation)
                                })
                        })
                        .collect::<Vec<_>>();

                    let signature = FunctionType {
                        params: parameters,
                        return_type: Box::new(
                            return_type
                                .as_ref()
                                .map(Type::from_type_expr)
                                .unwrap_or(Type::Dynamic),
                        ),
                    };

                    self.declare_global_declaration(name, Type::Function(signature.clone()))?;
                    self.functions.insert(name.clone(), signature);
                }

                Statement::Class {
                    name,
                    bases,
                    methods,
                } => {
                    let mut method_map = HashMap::new();

                    for method in methods {
                        let params = method
                            .params
                            .iter()
                            .enumerate()
                            .map(|(index, _)| {
                                method
                                    .param_types
                                    .get(index)
                                    .and_then(|annotation| annotation.as_ref())
                                    .map_or(Type::Dynamic, |annotation| {
                                        Type::from_type_expr(annotation)
                                    })
                            })
                            .collect::<Vec<_>>();

                        let return_type = method
                            .return_type
                            .as_ref()
                            .map(Type::from_type_expr)
                            .unwrap_or(Type::Dynamic);

                        method_map.insert(
                            method.name.clone(),
                            FunctionType {
                                params,
                                return_type: Box::new(return_type),
                            },
                        );
                    }

                    self.parents.insert(name.clone(), bases.clone());
                    self.classes.insert(
                        name.clone(),
                        ClassInfo {
                            bases: bases.clone(),
                            methods: method_map,
                        },
                    );
                    self.declare_global_declaration(name, Type::Named(name.clone()))?;
                }

                Statement::Interface { name, bases, .. } => {
                    self.parents.insert(name.clone(), bases.clone());
                    self.declare_global_declaration(name, Type::Named(name.clone()))?;
                }

                _ => {}
            }
        }

        Ok(())
    }

    fn strip_position(mut statement: &Statement) -> &Statement {
        while let Statement::Positioned {
            statement: inner, ..
        } = statement
        {
            statement = &**inner;
        }
        statement
    }

    fn declare_global_declaration(&mut self, name: &str, ty: Type) -> Result<(), CompileError> {
        let existing_native = self
            .scopes
            .first()
            .and_then(|scope| scope.get(name))
            .is_some_and(|binding| binding.native);

        if let Some(existing) = self.scopes.first().and_then(|scope| scope.get(name)) {
            if !existing.native || !existing_native {
                return Err(CompileError::VariableAlreadyDeclared(name.to_string()));
            }
        }

        self.scopes[0].insert(
            name.to_string(),
            Binding {
                ty,
                _mutable: true,
                native: false,
            },
        );
        Ok(())
    }

    fn check_statements(&mut self, statements: &[Statement]) -> Result<(), CompileError> {
        for statement in statements {
            self.check_statement(statement)?;
        }
        Ok(())
    }

    fn check_statement(&mut self, statement: &Statement) -> Result<(), CompileError> {
        match statement {
            Statement::Positioned {
                line,
                column,
                statement,
            } => self.with_location(*line, *column, |checker| checker.check_statement(statement)),

            Statement::Let {
                name,
                value,
                mutable,
                type_annotation,
            } => {
                let actual = self.check_expression(value)?;
                let declared = type_annotation
                    .as_ref()
                    .map(Type::from_type_expr)
                    .unwrap_or_else(|| actual.clone());

                self.ensure_assignable(&actual, &declared)?;

                self.declare(
                    name,
                    Binding {
                        ty: declared,
                        _mutable: *mutable,
                        native: false,
                    },
                )
            }

            Statement::Assignment { target, value } => {
                let actual = self.check_expression(value)?;
                self.check_assignment_target(target, &actual)
            }

            Statement::Expression { expression } => {
                self.check_expression(expression)?;
                Ok(())
            }

            Statement::Block(statements) => {
                self.push_scope();
                let result = self.check_statements(statements);
                self.pop_scope();
                result
            }

            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.check_expression(condition)?;
                self.push_scope();
                self.check_statements(then_branch)?;
                self.pop_scope();

                if let Some(else_branch) = else_branch {
                    self.push_scope();
                    self.check_statements(else_branch)?;
                    self.pop_scope();
                }

                Ok(())
            }

            Statement::While { condition, body } => {
                self.check_expression(condition)?;
                self.push_scope();
                let result = self.check_statements(body);
                self.pop_scope();
                result
            }

            Statement::ForIn {
                variable,
                iterable,
                body,
            } => {
                let iterable_type = self.check_expression(iterable)?;
                let element_type = iterable_type.element_type();

                self.push_scope();
                self.declare(
                    variable,
                    Binding {
                        ty: element_type,
                        _mutable: true,
                        native: false,
                    },
                )?;
                let result = self.check_statements(body);
                self.pop_scope();
                result
            }

            Statement::Match { value, arms } => {
                let value_type = self.check_expression(value)?;

                for arm in arms {
                    self.push_scope();
                    self.bind_pattern(&arm.pattern, &value_type)?;

                    if let Some(guard) = &arm.guard {
                        self.check_expression(guard)?;
                    }

                    self.check_statements(&arm.body)?;
                    self.pop_scope();
                }

                Ok(())
            }

            Statement::Throw { value } => {
                self.check_expression(value)?;
                Ok(())
            }

            Statement::Try {
                try_body,
                catch_name,
                catch_body,
                finally_body,
            } => {
                self.push_scope();
                self.check_statements(try_body)?;
                self.pop_scope();

                if let Some(catch_body) = catch_body {
                    self.push_scope();
                    if let Some(name) = catch_name {
                        self.declare(
                            name,
                            Binding {
                                ty: Type::Dynamic,
                                _mutable: true,
                                native: false,
                            },
                        )?;
                    }
                    self.check_statements(catch_body)?;
                    self.pop_scope();
                }

                if let Some(finally_body) = finally_body {
                    self.push_scope();
                    self.check_statements(finally_body)?;
                    self.pop_scope();
                }

                Ok(())
            }

            Statement::Function {
                name,
                params,
                param_types,
                return_type,
                body,
            } => self.check_function(name, params, param_types, return_type.as_ref(), body),

            Statement::Return { value } => {
                let actual = value
                    .as_ref()
                    .map(|expression| self.check_expression(expression))
                    .transpose()?
                    .unwrap_or(Type::None);

                if let Some(expected) = &self.current_return_type {
                    self.ensure_assignable(&actual, expected)?;
                }

                self.return_types.push(actual);
                Ok(())
            }

            Statement::Import { path } => self.check_import(path),

            Statement::FromImport { module, items } => self.check_from_import(module, items),

            Statement::Export { statement } => self.check_statement(statement),

            Statement::Class { name, methods, .. } => self.check_class(name, methods),

            Statement::Interface { .. } => Ok(()),

            Statement::Break | Statement::Continue => Ok(()),
        }
    }

    fn check_import(&mut self, path: &[String]) -> Result<(), CompileError> {
        let imported = self.resolve_import(path)?;
        let binding_name = path.last().ok_or(CompileError::InvalidImport)?;

        self.declare_import_binding(binding_name, imported)
    }

    fn check_from_import(
        &mut self,
        module: &ModulePath,
        items: &[ImportItem],
    ) -> Result<(), CompileError> {
        let context = self.context.as_ref().ok_or(CompileError::InvalidImport)?;
        let module_path = context
            .module_loader
            .resolver()
            .resolve(&context.current_module, &module.parts)?;
        let interface = context.module_loader.interface(&module_path)?;

        if items.len() == 1 && items[0].name == "*" {
            for (name, ty) in &interface.exports {
                self.declare_import_binding(name, ImportedType::Export(ty.clone()))?;
            }
            return Ok(());
        }

        for item in items {
            if item.name == "*" {
                return Err(CompileError::InvalidImport);
            }

            let ty = interface.exports.get(&item.name).cloned().ok_or_else(|| {
                CompileError::ExportNotFound {
                    module: module.parts.join("."),
                    name: item.name.clone(),
                }
            })?;

            let binding_name = item.alias.as_deref().unwrap_or(&item.name);
            self.declare_import_binding(binding_name, ImportedType::Export(ty))?;
        }

        Ok(())
    }

    fn resolve_import(&self, path: &[String]) -> Result<ImportedType, CompileError> {
        let context = self.context.as_ref().ok_or(CompileError::InvalidImport)?;
        context
            .module_loader
            .resolve_import(&context.current_module, path)
    }

    fn declare_import_binding(
        &mut self,
        binding_name: &str,
        imported: ImportedType,
    ) -> Result<(), CompileError> {
        let ty = match imported {
            ImportedType::Module(path) => Type::Module(path.to_string_lossy().into_owned()),
            ImportedType::Export(ty) => ty,
        };

        let is_global_scope = self.scopes.len() == 1;
        let native_collision = self
            .scopes
            .last()
            .and_then(|scope| scope.get(binding_name))
            .is_some_and(|binding| binding.native);

        if is_global_scope && native_collision {
            let scope = self
                .scopes
                .last_mut()
                .expect("TypeChecker scope is never empty");
            scope.insert(
                binding_name.to_string(),
                Binding {
                    ty,
                    _mutable: false,
                    native: false,
                },
            );
            return Ok(());
        }

        self.declare(
            binding_name,
            Binding {
                ty,
                _mutable: false,
                native: false,
            },
        )
    }

    fn export_type(&self, statement: &Statement) -> Result<(String, Type), CompileError> {
        match statement {
            Statement::Let { name, .. } => self
                .lookup(name)
                .map(|binding| (name.clone(), binding.ty))
                .ok_or_else(|| CompileError::UndefinedVariable {
                    name: name.clone(),
                    suggestion: None,
                }),

            Statement::Function { name, .. } => self
                .functions
                .get(name)
                .cloned()
                .map(|signature| (name.clone(), Type::Function(signature)))
                .ok_or_else(|| CompileError::UndefinedVariable {
                    name: name.clone(),
                    suggestion: None,
                }),

            Statement::Class { name, .. } | Statement::Interface { name, .. } => {
                Ok((name.clone(), Type::Named(name.clone())))
            }

            _ => Err(CompileError::InvalidExport),
        }
    }

    fn check_function(
        &mut self,
        name: &str,
        params: &[String],
        param_types: &[Option<TypeExpr>],
        return_type: Option<&TypeExpr>,
        body: &[Statement],
    ) -> Result<(), CompileError> {
        let previous_return = self.current_return_type.take();
        let previous_returns = std::mem::take(&mut self.return_types);

        let declared_signature = FunctionType {
            params: params
                .iter()
                .enumerate()
                .map(|(index, _)| {
                    param_types
                        .get(index)
                        .and_then(|annotation| annotation.as_ref())
                        .map_or(Type::Dynamic, |annotation| Type::from_type_expr(annotation))
                })
                .collect(),
            return_type: Box::new(
                return_type
                    .map(Type::from_type_expr)
                    .unwrap_or(Type::Dynamic),
            ),
        };

        let nested = self.scopes.len() > 1;
        let parent_scope_index = self.scopes.len() - 1;
        if nested {
            self.declare(
                name,
                Binding {
                    ty: Type::Function(declared_signature.clone()),
                    _mutable: true,
                    native: false,
                },
            )?;
        }

        self.push_scope();

        for (index, parameter) in params.iter().enumerate() {
            let ty = param_types
                .get(index)
                .and_then(|annotation| annotation.as_ref())
                .map_or(Type::Dynamic, |annotation| Type::from_type_expr(annotation));

            self.declare(
                parameter,
                Binding {
                    ty,
                    _mutable: true,
                    native: false,
                },
            )?;
        }

        self.current_return_type = return_type.map(Type::from_type_expr);
        self.check_statements(body)?;

        let inferred_return = self.infer_return_type();
        let missing_annotated_return = self
            .current_return_type
            .clone()
            .filter(|_| self.return_types.is_empty());

        if let Some(expected) = missing_annotated_return {
            if self.return_types.is_empty() {
                self.pop_scope();
                self.current_return_type = previous_return;
                self.return_types = previous_returns;

                return Err(CompileError::TypeMismatch {
                    expected: expected.to_string(),
                    found: Type::None.to_string(),
                });
            }
        }

        if return_type.is_none() {
            if let Some(function) = self.functions.get_mut(name) {
                function.return_type = Box::new(inferred_return.clone());
            }

            if nested {
                if let Some(binding) = self.scopes[parent_scope_index].get_mut(name) {
                    if let Type::Function(signature) = &mut binding.ty {
                        signature.return_type = Box::new(inferred_return);
                    }
                }
            }
        }

        self.pop_scope();
        self.current_return_type = previous_return;
        self.return_types = previous_returns;

        Ok(())
    }

    fn check_class(
        &mut self,
        class_name: &str,
        methods: &[FunctionMethod],
    ) -> Result<(), CompileError> {
        let previous_class = self.current_class.take();
        self.current_class = Some(class_name.to_string());

        for method in methods {
            self.check_method(class_name, method)?;
        }

        self.current_class = previous_class;
        Ok(())
    }

    fn check_method(
        &mut self,
        class_name: &str,
        method: &FunctionMethod,
    ) -> Result<(), CompileError> {
        let previous_return = self.current_return_type.take();
        let previous_returns = std::mem::take(&mut self.return_types);

        self.push_scope();

        self.declare(
            "this",
            Binding {
                ty: Type::Named(class_name.to_string()),
                _mutable: true,
                native: false,
            },
        )?;

        for (index, parameter) in method.params.iter().enumerate() {
            let ty = method
                .param_types
                .get(index)
                .and_then(|annotation| annotation.as_ref())
                .map_or(Type::Dynamic, |annotation| Type::from_type_expr(annotation));

            self.declare(
                parameter,
                Binding {
                    ty,
                    _mutable: true,
                    native: false,
                },
            )?;
        }

        self.current_return_type = method.return_type.as_ref().map(Type::from_type_expr);
        self.check_statements(&method.body)?;

        let inferred_return = self.infer_return_type();
        let missing_annotated_return = self
            .current_return_type
            .clone()
            .filter(|_| self.return_types.is_empty());

        if let Some(expected) = missing_annotated_return {
            if self.return_types.is_empty() {
                self.pop_scope();
                self.current_return_type = previous_return;
                self.return_types = previous_returns;

                return Err(CompileError::TypeMismatch {
                    expected: expected.to_string(),
                    found: Type::None.to_string(),
                });
            }
        }

        if let Some(class) = self.classes.get_mut(class_name) {
            if let Some(signature) = class.methods.get_mut(&method.name) {
                if method.return_type.is_none() {
                    signature.return_type = Box::new(inferred_return);
                }
            }
        }

        self.pop_scope();
        self.current_return_type = previous_return;
        self.return_types = previous_returns;

        Ok(())
    }

    fn infer_return_type(&self) -> Type {
        self.return_types
            .iter()
            .cloned()
            .reduce(|left, right| left.merge(&right))
            .unwrap_or(Type::None)
    }

    fn check_assignment_target(
        &mut self,
        target: &AssignmentTarget,
        actual: &Type,
    ) -> Result<(), CompileError> {
        match target {
            AssignmentTarget::Variable(name) => {
                if let Some(binding) = self.lookup(name) {
                    let expected = binding.ty.clone();
                    self.ensure_assignable(actual, &expected)
                } else {
                    Ok(())
                }
            }

            AssignmentTarget::Index { object, index } => {
                let object_type = self.check_expression(object)?;
                let index_type = self.check_expression(index)?;

                match object_type {
                    Type::Array(_) | Type::ArrayDynamic | Type::Tuple(_) | Type::TupleDynamic => {
                        if !matches!(index_type, Type::Int | Type::Float | Type::Dynamic) {
                            return Err(CompileError::TypeMismatch {
                                expected: "int".to_string(),
                                found: index_type.to_string(),
                            });
                        }

                        let expected = object_type.element_type();
                        self.ensure_assignable(actual, &expected)
                    }

                    Type::Dict(key, value) => {
                        self.ensure_assignable(&index_type, &key)?;
                        self.ensure_assignable(actual, &value)
                    }

                    Type::DictDynamic => Ok(()),
                    Type::Dynamic => Ok(()),
                    _ => Ok(()),
                }
            }

            AssignmentTarget::Member { object, .. } => {
                self.check_expression(object)?;
                Ok(())
            }
        }
    }

    fn check_expression(&mut self, expression: &Expression) -> Result<Type, CompileError> {
        match expression {
            Expression::Literal(literal) => Ok(match literal {
                Literal::Integer(_) => Type::Int,
                Literal::Float(_) => Type::Float,
                Literal::String(_) => Type::Str,
                Literal::Bool(_) => Type::Bool,
                Literal::None => Type::None,
            }),

            Expression::Variable(name) => Ok(self
                .lookup(name)
                .map(|binding| binding.ty)
                .or_else(|| self.functions.get(name).cloned().map(Type::Function))
                .unwrap_or(Type::Dynamic)),

            Expression::Unary {
                operator, right, ..
            } => {
                let right_type = self.check_expression(right)?;

                match operator {
                    UnaryOp::Not => Ok(Type::Bool),
                    UnaryOp::Negate => {
                        if right_type.is_dynamic() {
                            Ok(Type::Dynamic)
                        } else if right_type.numeric_kind().is_some() {
                            Ok(right_type)
                        } else {
                            Err(CompileError::InvalidUnaryOperation {
                                operator: "-".to_string(),
                                found: right_type.to_string(),
                            })
                        }
                    }
                    UnaryOp::BitNot => {
                        if right_type.is_dynamic() || right_type == Type::Int {
                            Ok(Type::Int)
                        } else {
                            Err(CompileError::InvalidUnaryOperation {
                                operator: "~".to_string(),
                                found: right_type.to_string(),
                            })
                        }
                    }
                }
            }

            Expression::Binary {
                left,
                operator,
                right,
                ..
            } => self.check_binary(left, operator, right),

            Expression::Function { params, body } => {
                let previous_return = self.current_return_type.take();
                let previous_returns = std::mem::take(&mut self.return_types);

                self.push_scope();
                for parameter in params {
                    self.declare(
                        parameter,
                        Binding {
                            ty: Type::Dynamic,
                            _mutable: true,
                            native: false,
                        },
                    )?;
                }

                self.current_return_type = None;
                self.check_statements(body)?;
                let return_type = self.infer_return_type();
                self.pop_scope();

                self.current_return_type = previous_return;
                self.return_types = previous_returns;

                Ok(Type::Function(FunctionType {
                    params: vec![Type::Dynamic; params.len()],
                    return_type: Box::new(return_type),
                }))
            }

            Expression::Call {
                callee, arguments, ..
            } => {
                let callee_type = self.check_expression(callee)?;

                match callee_type {
                    Type::Function(signature) => {
                        if signature.params.len() != arguments.len() {
                            return Err(CompileError::WrongArgumentCount {
                                expected: signature.params.len() as i32,
                                found: arguments.len(),
                            });
                        }

                        for (index, (argument, expected)) in
                            arguments.iter().zip(&signature.params).enumerate()
                        {
                            let actual = self.check_expression(argument)?;

                            if !self.are_assignable(&actual, expected) {
                                return Err(CompileError::WrongArgumentType {
                                    function: self.expression_name(callee),
                                    index: index + 1,
                                    expected: expected.to_string(),
                                    found: actual.to_string(),
                                });
                            }
                        }

                        Ok(*signature.return_type)
                    }

                    Type::Dynamic => {
                        for argument in arguments {
                            self.check_expression(argument)?;
                        }
                        Ok(Type::Dynamic)
                    }

                    other => Err(CompileError::NotCallable {
                        found: other.to_string(),
                    }),
                }
            }

            Expression::Array(elements) => {
                if elements.is_empty() {
                    return Ok(Type::Array(Box::new(Type::Dynamic)));
                }

                let mut element_type = self.check_expression(&elements[0])?;
                for element in &elements[1..] {
                    let next = self.check_expression(element)?;
                    element_type = element_type.merge(&next);
                }
                Ok(Type::Array(Box::new(element_type)))
            }

            Expression::Tuple(elements) => {
                let mut types = Vec::with_capacity(elements.len());
                for element in elements {
                    types.push(self.check_expression(element)?);
                }
                Ok(Type::Tuple(types))
            }

            Expression::Object(fields) => {
                if fields.is_empty() {
                    return Ok(Type::Dict(Box::new(Type::Str), Box::new(Type::Dynamic)));
                }

                let mut value_type = self.check_expression(&fields[0].1)?;
                for (_, value) in &fields[1..] {
                    value_type = value_type.merge(&self.check_expression(value)?);
                }

                Ok(Type::Dict(Box::new(Type::Str), Box::new(value_type)))
            }

            Expression::Index { object, index, .. } => {
                let object_type = self.check_expression(object)?;
                let index_type = self.check_expression(index)?;

                match object_type {
                    Type::Array(_) | Type::ArrayDynamic | Type::Tuple(_) | Type::TupleDynamic => {
                        if !matches!(index_type, Type::Int | Type::Float | Type::Dynamic) {
                            return Err(CompileError::TypeMismatch {
                                expected: "int".to_string(),
                                found: index_type.to_string(),
                            });
                        }
                        Ok(object_type.element_type())
                    }
                    Type::Dict(key, value) => {
                        self.ensure_assignable(&index_type, &key)?;
                        Ok(*value)
                    }
                    Type::DictDynamic | Type::Dynamic => Ok(Type::Dynamic),
                    _ => Ok(Type::Dynamic),
                }
            }

            Expression::Member { object, name, .. } => {
                let object_type = self.check_expression(object)?;
                self.member_type(&object_type, name)
            }

            Expression::New {
                class_name,
                arguments,
                ..
            } => {
                if let Some(signature) = self.find_method(class_name, "init") {
                    if signature.params.len() != arguments.len() {
                        return Err(CompileError::WrongArgumentCount {
                            expected: signature.params.len() as i32,
                            found: arguments.len(),
                        });
                    }

                    for (index, (argument, expected)) in
                        arguments.iter().zip(&signature.params).enumerate()
                    {
                        let actual = self.check_expression(argument)?;
                        if !self.are_assignable(&actual, expected) {
                            return Err(CompileError::WrongArgumentType {
                                function: format!("{class_name}.init"),
                                index: index + 1,
                                expected: expected.to_string(),
                                found: actual.to_string(),
                            });
                        }
                    }
                } else {
                    for argument in arguments {
                        self.check_expression(argument)?;
                    }
                }

                Ok(Type::Named(class_name.clone()))
            }

            Expression::This => Ok(self
                .current_class
                .as_ref()
                .map(|name| Type::Named(name.clone()))
                .unwrap_or(Type::Dynamic)),

            Expression::Base => Ok(Type::Dynamic),

            Expression::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                self.check_expression(condition)?;
                let then_type = self.check_expression(then_expr)?;
                let else_type = self.check_expression(else_expr)?;
                Ok(then_type.merge(&else_type))
            }
        }
    }

    fn check_binary(
        &mut self,
        left: &Expression,
        operator: &BinaryOp,
        right: &Expression,
    ) -> Result<Type, CompileError> {
        let left_type = self.check_expression(left)?;
        let right_type = self.check_expression(right)?;

        match operator {
            BinaryOp::And | BinaryOp::Or => Ok(left_type.merge(&right_type)),

            BinaryOp::Equal | BinaryOp::NotEqual | BinaryOp::Is => Ok(Type::Bool),

            BinaryOp::Add => {
                if left_type.is_dynamic() && right_type == Type::Str {
                    return Ok(Type::Str);
                }

                if right_type.is_dynamic() && left_type == Type::Str {
                    return Ok(Type::Str);
                }

                if left_type.is_dynamic() {
                    if right_type.numeric_kind().is_some() {
                        return Ok(right_type);
                    }
                    return Ok(Type::Dynamic);
                }

                if right_type.is_dynamic() {
                    if left_type.numeric_kind().is_some() {
                        return Ok(left_type);
                    }
                    return Ok(Type::Dynamic);
                }

                if left_type == Type::Str && right_type == Type::Str {
                    return Ok(Type::Str);
                }

                if let (Some(left), Some(right)) =
                    (left_type.numeric_kind(), right_type.numeric_kind())
                {
                    return Ok(match (left, right) {
                        (super::types::NumericType::Int, super::types::NumericType::Int) => {
                            Type::Int
                        }
                        _ => Type::Float,
                    });
                }

                Err(CompileError::InvalidBinaryOperation {
                    operator: "+".to_string(),
                    left: left_type.to_string(),
                    right: right_type.to_string(),
                })
            }

            BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => {
                if left_type.is_dynamic() {
                    if right_type.numeric_kind().is_some() {
                        return Ok(right_type);
                    }
                    return Ok(Type::Dynamic);
                }

                if right_type.is_dynamic() {
                    if left_type.numeric_kind().is_some() {
                        return Ok(left_type);
                    }
                    return Ok(Type::Dynamic);
                }

                if let (Some(left), Some(right)) =
                    (left_type.numeric_kind(), right_type.numeric_kind())
                {
                    return Ok(match operator {
                        BinaryOp::Divide => Type::Float,
                        BinaryOp::Modulo => match (left, right) {
                            (super::types::NumericType::Int, super::types::NumericType::Int) => {
                                Type::Int
                            }
                            _ => Type::Float,
                        },
                        _ => match (left, right) {
                            (super::types::NumericType::Int, super::types::NumericType::Int) => {
                                Type::Int
                            }
                            _ => Type::Float,
                        },
                    });
                }

                Err(CompileError::InvalidBinaryOperation {
                    operator: binary_symbol(operator).to_string(),
                    left: left_type.to_string(),
                    right: right_type.to_string(),
                })
            }

            BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual => {
                if left_type.is_dynamic() || right_type.is_dynamic() {
                    return Ok(Type::Bool);
                }

                if left_type.numeric_kind().is_some() && right_type.numeric_kind().is_some() {
                    return Ok(Type::Bool);
                }

                Err(CompileError::InvalidBinaryOperation {
                    operator: binary_symbol(operator).to_string(),
                    left: left_type.to_string(),
                    right: right_type.to_string(),
                })
            }

            BinaryOp::BitAnd
            | BinaryOp::BitOr
            | BinaryOp::BitXor
            | BinaryOp::ShiftLeft
            | BinaryOp::ShiftRight => {
                if left_type.is_dynamic() || right_type.is_dynamic() {
                    return Ok(Type::Int);
                }

                if left_type == Type::Int && right_type == Type::Int {
                    return Ok(Type::Int);
                }

                Err(CompileError::InvalidBinaryOperation {
                    operator: binary_symbol(operator).to_string(),
                    left: left_type.to_string(),
                    right: right_type.to_string(),
                })
            }
        }
    }

    fn member_type(&self, object_type: &Type, name: &str) -> Result<Type, CompileError> {
        match object_type {
            Type::Named(class_name) => {
                if let Some(signature) = self.find_method(class_name, name) {
                    return Ok(Type::Function(signature));
                }
            }

            Type::Module(path) => {
                let context =
                    self.context
                        .as_ref()
                        .ok_or_else(|| CompileError::InvalidMemberAccess {
                            name: name.to_string(),
                        })?;
                let interface = context
                    .module_loader
                    .interface(std::path::Path::new(path))?;
                return interface.exports.get(name).cloned().ok_or_else(|| {
                    CompileError::InvalidMemberAccess {
                        name: name.to_string(),
                    }
                });
            }

            _ => {}
        }

        Ok(Type::Dynamic)
    }

    fn find_method(&self, class_name: &str, method_name: &str) -> Option<FunctionType> {
        let mut pending = vec![class_name.to_string()];
        let mut visited = HashSet::new();

        while let Some(name) = pending.pop() {
            if !visited.insert(name.clone()) {
                continue;
            }

            if let Some(class) = self.classes.get(&name) {
                if let Some(signature) = class.methods.get(method_name) {
                    return Some(signature.clone());
                }

                pending.extend(class.bases.iter().cloned());
            }
        }

        None
    }

    fn bind_pattern(&mut self, pattern: &Pattern, matched_type: &Type) -> Result<(), CompileError> {
        match pattern {
            Pattern::Wildcard | Pattern::Literal(_) | Pattern::Range { .. } => Ok(()),
            Pattern::Binding(name) => self.declare(
                name,
                Binding {
                    ty: matched_type.clone(),
                    _mutable: true,
                    native: false,
                },
            ),
            Pattern::Or(patterns) => {
                for pattern in patterns {
                    self.bind_pattern(pattern, matched_type)?;
                }
                Ok(())
            }
            Pattern::Array(patterns) => {
                let element_type = matched_type.element_type();
                for pattern in patterns {
                    self.bind_pattern(pattern, &element_type)?;
                }
                Ok(())
            }
        }
    }

    fn expression_name(&self, expression: &Expression) -> String {
        match expression {
            Expression::Variable(name) => name.clone(),
            Expression::Member { object, name, .. } => {
                let object = self.expression_name(object);
                format!("{object}.{name}")
            }
            _ => "<call>".to_string(),
        }
    }

    fn are_assignable(&self, actual: &Type, expected: &Type) -> bool {
        let parents = |name: &str| self.parents.get(name).cloned().unwrap_or_default();
        actual.is_assignable_to(expected, &parents)
    }

    fn ensure_assignable(&self, actual: &Type, expected: &Type) -> Result<(), CompileError> {
        if self.are_assignable(actual, expected) {
            Ok(())
        } else {
            Err(CompileError::TypeMismatch {
                expected: expected.to_string(),
                found: actual.to_string(),
            })
        }
    }

    fn declare(&mut self, name: &str, binding: Binding) -> Result<(), CompileError> {
        let is_global_scope = self.scopes.len() == 1;
        let existing_native = self
            .scopes
            .last()
            .and_then(|scope| scope.get(name))
            .is_some_and(|binding| binding.native);

        let scope = self
            .scopes
            .last_mut()
            .expect("TypeChecker scope is never empty");

        if scope.contains_key(name) {
            if is_global_scope && existing_native {
                scope.insert(
                    name.to_string(),
                    Binding {
                        native: false,
                        ..binding
                    },
                );
                return Ok(());
            }

            return Err(CompileError::VariableAlreadyDeclared(name.to_string()));
        }

        scope.insert(name.to_string(), binding);
        Ok(())
    }

    fn lookup(&self, name: &str) -> Option<Binding> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned())
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        debug_assert!(self.scopes.len() > 1);
        self.scopes.pop();
    }

    fn with_location<T>(
        &mut self,
        line: usize,
        column: usize,
        f: impl FnOnce(&mut Self) -> Result<T, CompileError>,
    ) -> Result<T, CompileError> {
        f(&mut *self).map_err(|error| match error {
            CompileError::WithLocation { .. } => error,
            other => CompileError::WithLocation {
                line,
                column,
                source: Box::new(other),
            },
        })
    }
}

fn binary_symbol(operator: &BinaryOp) -> &'static str {
    match operator {
        BinaryOp::Add => "+",
        BinaryOp::Subtract => "-",
        BinaryOp::Multiply => "*",
        BinaryOp::Divide => "/",
        BinaryOp::Modulo => "%",
        BinaryOp::Equal => "==",
        BinaryOp::NotEqual => "!=",
        BinaryOp::Less => "<",
        BinaryOp::LessEqual => "<=",
        BinaryOp::Greater => ">",
        BinaryOp::GreaterEqual => ">=",
        BinaryOp::And => "and",
        BinaryOp::Or => "or",
        BinaryOp::Is => "is",
        BinaryOp::BitAnd => "&",
        BinaryOp::BitOr => "|",
        BinaryOp::BitXor => "^",
        BinaryOp::ShiftLeft => "<<",
        BinaryOp::ShiftRight => ">>",
    }
}

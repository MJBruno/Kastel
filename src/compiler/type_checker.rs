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
    /// Une classe peut surcharger une méthode par son arité.
    /// Deux signatures de même nom et de même arité restent interdites.
    methods: HashMap<String, Vec<FunctionType>>,
    /// Champs déclarés par `let nom: type = ...;` dans le corps de la classe.
    fields: HashMap<String, Type>,
    /// Membres (champs et méthodes) déclarés `private` dans cette classe.
    private_members: HashSet<String>,
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
                    fields,
                    methods,
                } => {
                    let mut method_map: HashMap<String, Vec<FunctionType>> = HashMap::new();
                    let mut field_map: HashMap<String, Type> = HashMap::new();
                    let mut private_members: HashSet<String> = HashSet::new();

                    for field in fields {
                        field_map.insert(
                            field.name.clone(),
                            field
                                .type_annotation
                                .as_ref()
                                .map(Type::from_type_expr)
                                .unwrap_or(Type::Dynamic),
                        );

                        if field.visibility == Visibility::Private {
                            private_members.insert(field.name.clone());
                        }
                    }

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

                        let signature = FunctionType {
                            params,
                            return_type: Box::new(return_type),
                        };

                        let overloads = method_map.entry(method.name.clone()).or_default();

                        if overloads
                            .iter()
                            .any(|existing| existing.params.len() == signature.params.len())
                        {
                            return Err(CompileError::DuplicateMethod {
                                class_name: name.clone(),
                                method_name: method.name.clone(),
                                arity: signature.params.len(),
                            });
                        }

                        overloads.push(signature);

                        if method.visibility == Visibility::Private {
                            private_members.insert(method.name.clone());
                        }
                    }

                    self.parents.insert(name.clone(), bases.clone());
                    self.classes.insert(
                        name.clone(),
                        ClassInfo {
                            bases: bases.clone(),
                            methods: method_map,
                            fields: field_map,
                            private_members,
                        },
                    );
                    self.declare_global_declaration(name, Type::Named(name.clone()))?;
                }

                Statement::Interface {
                    name,
                    bases,
                    methods,
                } => {
                    // Une interface se surcharge comme une classe : le
                    // couple (nom, arité) doit rester unique.
                    let mut method_map: HashMap<String, Vec<FunctionType>> = HashMap::new();

                    for method in methods {
                        let params = (0..method.arity)
                            .map(|index| {
                                method
                                    .param_types
                                    .get(index)
                                    .and_then(|annotation| annotation.as_ref())
                                    .map_or(Type::Dynamic, Type::from_type_expr)
                            })
                            .collect::<Vec<_>>();

                        let signature = FunctionType {
                            params,
                            return_type: Box::new(
                                method
                                    .return_type
                                    .as_ref()
                                    .map(Type::from_type_expr)
                                    .unwrap_or(Type::Dynamic),
                            ),
                        };

                        let overloads = method_map.entry(method.name.clone()).or_default();

                        if overloads
                            .iter()
                            .any(|existing| existing.params.len() == signature.params.len())
                        {
                            return Err(CompileError::DuplicateMethod {
                                class_name: name.clone(),
                                method_name: method.name.clone(),
                                arity: signature.params.len(),
                            });
                        }

                        overloads.push(signature);
                    }

                    self.parents.insert(name.clone(), bases.clone());
                    // Enregistrée comme « classe sans corps » : les appels sur
                    // une valeur typée par l'interface sont ainsi vérifiés
                    // (arité + types) via `find_methods`.
                    self.classes.insert(
                        name.clone(),
                        ClassInfo {
                            bases: bases.clone(),
                            methods: method_map,
                            fields: HashMap::new(),
                            private_members: HashSet::new(),
                        },
                    );
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
            if let Some(overloads) = class.methods.get_mut(&method.name) {
                if let Some(signature) = overloads
                    .iter_mut()
                    .find(|signature| signature.params.len() == method.params.len())
                    && method.return_type.is_none()
                {
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

            AssignmentTarget::Member { object, name } => {
                let object_type = self.check_expression(object)?;

                if let Type::Named(class_name) = &object_type {
                    self.check_member_visibility(class_name, name)?;

                    // `this.age = valeur` : la valeur doit respecter le type
                    // déclaré du champ (`let age: int`).
                    if let Some(expected) = self.find_field(class_name, name) {
                        self.ensure_assignable(actual, &expected)?;
                    }
                }

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
                // Pour une méthode de classe, l'arité fait partie de la
                // résolution. Cela permet `obj.foo()` et `obj.foo(x)`
                // d'aboutir à deux signatures différentes.
                if let Expression::Member { object, name, .. } = callee.as_ref() {
                    let object_type = self.check_expression(object)?;

                    if let Type::Named(class_name) = object_type {
                        self.check_member_visibility(&class_name, name)?;

                        let signatures = self.find_methods(&class_name, name);

                        if !signatures.is_empty() {
                            let signature = self.resolve_overload(
                                &signatures,
                                arguments,
                                &format!("{class_name}.{name}"),
                            )?;

                            return Ok(*signature.return_type);
                        }
                    }
                }

                let callee_type = self.check_expression(callee)?;

                match callee_type {
                    Type::Function(signature) => {
                        let function_name = self.expression_name(callee);
                        self.check_call_signature(&signature, arguments, &function_name)
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
                        // `t[0]` sur un `Tuple<int, str>` : type EXACT de
                        // l'élément quand l'index est un littéral entier.
                        if let (
                            Type::Tuple(elements),
                            Expression::Literal(Literal::Integer(position)),
                        ) = (&object_type, index.as_ref())
                            && let Ok(position) = usize::try_from(*position)
                            && let Some(element) = elements.get(position)
                        {
                            return Ok(element.clone());
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
                let signatures = self.find_methods(class_name, "init");

                if !signatures.is_empty() {
                    self.resolve_overload(
                        &signatures,
                        arguments,
                        &format!("{class_name}.init"),
                    )?;
                } else {
                    // Classe sans constructeur connu : les arguments restent
                    // dynamiques pour conserver le typage graduel.
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
                // `/` renvoie TOUJOURS un flottant à l'exécution : même avec
                // un opérande dynamique, le résultat n'est jamais un `int`.
                let is_divide = matches!(operator, BinaryOp::Divide);

                if left_type.is_dynamic() {
                    if right_type.numeric_kind().is_some() {
                        return Ok(if is_divide { Type::Float } else { right_type });
                    }
                    return Ok(Type::Dynamic);
                }

                if right_type.is_dynamic() {
                    if left_type.numeric_kind().is_some() {
                        return Ok(if is_divide { Type::Float } else { left_type });
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
                self.check_member_visibility(class_name, name)?;

                let signatures = self.find_methods(class_name, name);

                if signatures.len() == 1 {
                    return Ok(Type::Function(signatures[0].clone()));
                }

                if signatures.len() > 1 {
                    // Une surcharge ne forme pas une valeur de fonction unique
                    // sans contexte d'appel. Les appels directs sont traités
                    // plus haut et sélectionnent la bonne signature.
                    return Ok(Type::Dynamic);
                }

                // Champ déclaré (`let age: int = 0;`) : son type est connu.
                if let Some(field_type) = self.find_field(class_name, name) {
                    return Ok(field_type);
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

    /// Première classe de la hiérarchie de `class_name` qui déclare `member`
    /// (champ ou méthode), avec sa visibilité (`true` = privé).
    fn find_member_declaration(&self, class_name: &str, member: &str) -> Option<(String, bool)> {
        let mut pending = vec![class_name.to_string()];
        let mut visited = HashSet::new();

        while let Some(name) = pending.pop() {
            if !visited.insert(name.clone()) {
                continue;
            }

            if let Some(class) = self.classes.get(&name) {
                if class.fields.contains_key(member) || class.methods.contains_key(member) {
                    return Some((name, class.private_members.contains(member)));
                }

                pending.extend(class.bases.iter().cloned());
            }
        }

        None
    }

    /// Refuse `objet.membre` si `membre` est privé et que le code courant
    /// n'est pas dans le corps de la classe qui le déclare.
    ///
    /// Ne voit que les classes connues de ce fichier : pour une classe
    /// importée (ou une valeur dynamique), c'est la VM qui contrôle.
    fn check_member_visibility(&self, class_name: &str, member: &str) -> Result<(), CompileError> {
        match self.find_member_declaration(class_name, member) {
            Some((owner, true)) if self.current_class.as_deref() != Some(owner.as_str()) => {
                Err(CompileError::PrivateMemberAccess {
                    class_name: owner,
                    member: member.to_string(),
                })
            }

            _ => Ok(()),
        }
    }

    /// Type déclaré d'un champ, en remontant la hiérarchie.
    fn find_field(&self, class_name: &str, field: &str) -> Option<Type> {
        let mut pending = vec![class_name.to_string()];
        let mut visited = HashSet::new();

        while let Some(name) = pending.pop() {
            if !visited.insert(name.clone()) {
                continue;
            }

            if let Some(class) = self.classes.get(&name) {
                if let Some(ty) = class.fields.get(field) {
                    return Some(ty.clone());
                }

                pending.extend(class.bases.iter().cloned());
            }
        }

        None
    }

    fn find_methods(&self, class_name: &str, method_name: &str) -> Vec<FunctionType> {
        let mut pending = vec![class_name.to_string()];
        let mut visited = HashSet::new();
        let mut seen_arities = HashSet::new();
        let mut result = Vec::new();

        while let Some(name) = pending.pop() {
            if !visited.insert(name.clone()) {
                continue;
            }

            if let Some(class) = self.classes.get(&name) {
                if let Some(overloads) = class.methods.get(method_name) {
                    for signature in overloads {
                        // Une surcharge définie dans la classe dérivée masque
                        // la signature de même arité d'une classe de base.
                        if seen_arities.insert(signature.params.len()) {
                            result.push(signature.clone());
                        }
                    }
                }

                pending.extend(class.bases.iter().cloned());
            }
        }

        result
    }

    fn check_call_signature(
        &mut self,
        signature: &FunctionType,
        arguments: &[Expression],
        function_name: &str,
    ) -> Result<Type, CompileError> {
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
                    function: function_name.to_string(),
                    index: index + 1,
                    expected: expected.to_string(),
                    found: actual.to_string(),
                });
            }
        }

        Ok((*signature.return_type).clone())
    }

    fn resolve_overload(
        &mut self,
        signatures: &[FunctionType],
        arguments: &[Expression],
        function_name: &str,
    ) -> Result<FunctionType, CompileError> {
        let signature = signatures
            .iter()
            .find(|signature| signature.params.len() == arguments.len());

        let Some(signature) = signature else {
            let expected = signatures
                .first()
                .map(|signature| signature.params.len() as i32)
                .unwrap_or(0);

            return Err(CompileError::WrongArgumentCount {
                expected,
                found: arguments.len(),
            });
        };

        self.check_call_signature(signature, arguments, function_name)?;

        Ok(signature.clone())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::{lexer::lexer::Lexer, parser::Parser};

    fn check(source: &str) -> Result<(), CompileError> {
        let tokens = Lexer::new(source.to_string()).scan_token().unwrap();
        let statements = Parser::new(tokens).parse().unwrap();
        TypeChecker::check(&statements)
    }

    #[test]
    fn inference_and_annotations_are_accepted() {
        let result = check(
            r#"
let x = 10;
let y: int = 20;
let z = x + y;
let name: str = "Bruno";
let values: Array<int> = [1, 2, 3];
let users: Dict<str, int> = {
    age: 25
};
func add(a: int, b: int) -> int {
    return a + b;
}
let result = add(10, 20);
"#,
        );
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn nested_generics_and_compact_equal_parse() {
        let result = check(
            "let m: Dict<str, Array<int>> = { a: [1, 2] };\nlet v: Array<int>= [1, 2];",
        );
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn wrong_annotation_is_rejected() {
        assert!(check("let x: int = \"a\";").is_err());
    }

    #[test]
    fn division_with_dynamic_operand_is_float() {
        assert!(check("func f(n) { let q: int = n / 2; return q; }").is_err());
        assert!(check("func g(n) { let q: float = n / 2; return q; }").is_ok());
    }

    #[test]
    fn methods_and_constructors_can_be_overloaded_by_arity() {
        let result = check(
            r#"
class Point {
    func init() { this.x = 0; this.y = 0; }
    func init(x: int) { this.x = x; this.y = 0; }
    func init(x: int, y: int) { this.x = x; this.y = y; }
    func scale() -> int { return 1; }
    func scale(k: int) -> int { return k; }
}
let a = new Point();
let b = new Point(1);
let c = new Point(1, 2);
let k: int = c.scale();
let m: int = c.scale(3);
"#,
        );
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn same_arity_method_is_a_duplicate() {
        let result = check("class A { func f(x) { return 1; } func f(y) { return 2; } }");
        assert!(matches!(result, Err(CompileError::DuplicateMethod { .. })));
    }

    #[test]
    fn no_overload_for_the_given_arity_is_rejected() {
        assert!(
            check("class A { func f(x) { return 1; } } let a = new A(); a.f(1, 2);").is_err()
        );
        assert!(
            check("class B { func init(x) { this.x = x; } } let b = new B();").is_err()
        );
    }

    #[test]
    fn interfaces_can_declare_overloads() {
        let result = check(
            r#"
interface Shape {
    func area() -> float;
    func area(scale: float) -> float;
}
class Square: Shape {
    func area() -> float { return 1.0; }
    func area(scale: float) -> float { return scale; }
}
let s: Shape = new Square();
let a: float = s.area();
let b: float = s.area(2.0);
"#,
        );
        assert!(result.is_ok(), "{:?}", result.err());

        let duplicate = check("interface I { func f(); func f(); }");
        assert!(matches!(duplicate, Err(CompileError::DuplicateMethod { .. })));

        let wrong_call = check(
            r#"
interface Shape { func area() -> float; }
class Square: Shape { func area() -> float { return 1.0; } }
let s: Shape = new Square();
s.area(1.0, 2.0);
"#,
        );
        assert!(wrong_call.is_err());
    }

    #[test]
    fn tuple_annotations_are_checked_element_by_element() {
        let result = check(
            r#"
let t: Tuple<float, str> = (1, "a");
func first(p: Tuple<int, int>) -> int { return p[0]; }
let r: int = first((1, 2));
"#,
        );
        assert!(result.is_ok(), "{:?}", result.err());

        assert!(check("let bad: Tuple<int> = (1, 2);").is_err());
        assert!(check("let bad: Tuple<int, str> = (1.5, \"a\");").is_err());
    }

    fn parse_fails(source: &str) -> bool {
        let tokens = Lexer::new(source.to_string()).scan_token().unwrap();
        Parser::new(tokens).parse().is_err()
    }

    /// Les erreurs relevées pendant `check_statements` sont enveloppées dans
    /// `WithLocation` : on remonte à l'erreur d'origine.
    fn is_private_access(result: &Result<(), CompileError>) -> bool {
        let mut error = match result {
            Err(error) => error,
            Ok(()) => return false,
        };

        while let CompileError::WithLocation { source, .. } = error {
            error = &**source;
        }

        matches!(error, CompileError::PrivateMemberAccess { .. })
    }

    const PERSONNE: &str = r#"
class Personne {
    private let age: int = 0;

    public func initialize(age: int) {
        this.age = age;
    }

    public func setAge(age: int) {
        this.age = age;
    }

    public func getAge() -> int {
        return this.age;
    }

    public func number() -> int {
        return 22;
    }
}

let p: Personne = new Personne(26);
p.setAge(44);
let age: int = p.getAge();
"#;

    #[test]
    fn public_api_of_a_class_with_private_field_type_checks() {
        let result = check(PERSONNE);
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn private_field_is_forbidden_outside_the_class() {
        let read = check(&format!("{PERSONNE}\nprintln(p.age);"));
        assert!(is_private_access(&read), "{:?}", read);

        let write = check(&format!("{PERSONNE}\np.age = 3;"));
        assert!(is_private_access(&write), "{:?}", write);
    }

    #[test]
    fn private_method_is_forbidden_outside_the_class() {
        let result = check(
            r#"
class A {
    private func secret() -> int { return 1; }
    func open() -> int { return this.secret(); }
}
let a = new A();
let x: int = a.open();
a.secret();
"#,
        );
        assert!(is_private_access(&result), "{:?}", result);
    }

    #[test]
    fn private_members_are_usable_by_other_instances_and_callbacks() {
        let result = check(
            r#"
class A {
    private let n: int = 1;
    func same(other: A) -> int { return other.n; }
    func later() { let f = func() { return this.n; }; return f(); }
}
"#,
        );
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn declared_field_types_are_enforced() {
        assert!(check("class A { let n: int = 0; func f() { this.n = \"x\"; } }").is_err());
        assert!(check("class A { let n: int = \"x\"; }").is_err());
        assert!(check("class A { let n: float = 1; }").is_ok());
    }

    #[test]
    fn initialize_is_an_alias_of_the_constructor() {
        assert!(
            check("class A { func initialize(x: int) { this.x = x; } } let a = new A(1);").is_ok()
        );
        assert!(
            check("class A { func initialize(x: int) { this.x = x; } } let a = new A();").is_err()
        );
    }

    #[test]
    fn duplicate_fields_and_mixed_overload_visibility_are_parse_errors() {
        assert!(parse_fails("class A { let n = 1; let n = 2; }"));
        assert!(parse_fails(
            "class A { func f() { return 1; } private func f(x) { return 2; } }"
        ));
    }
}


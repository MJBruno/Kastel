//! HIM — High-level Intermediate Model de Kastel.
//!
//! La HIM est la représentation intermédiaire possédée par le backend :
//! elle ne contient aucune référence vers `frontend::ast`.
//!
//! Pipeline :
//!
//! ```text
//! source -> lexer -> parser -> AST -> TypeChecker -> HIM -> bytecode -> VM
//! ```
//!
//! Le TypeChecker travaille encore sur l'AST (c'est sa responsabilité de
//! frontend). Une fois cette analyse terminée, l'AST est abaissé vers cette
//! représentation possédée. Le backend ne dépend ensuite plus du frontend.

use std::rc::Rc;

use crate::error::compile_error::CompileError;
use crate::frontend::ast as ast;

use super::call_metadata::{ResolvedCallTable, ResolvedMemberTable};
use super::type_checker::{TypeCheckContext, TypeChecker};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GenericParam {
    pub(crate) name: String,
    pub(crate) bounds: Vec<TypeExpr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TypeExpr {
    Named(String),
    Generic {
        name: String,
        arguments: Vec<TypeExpr>,
    },
    Union(Vec<TypeExpr>),
    Record(Vec<(String, TypeExpr)>),
}

#[derive(Debug, Clone)]
pub(crate) enum AssignmentTarget {
    Variable(String),
    Index {
        object: Box<Expression>,
        index: Box<Expression>,
    },
    Member {
        object: Box<Expression>,
        name: String,
    },
}

#[derive(Debug, Clone)]
pub(crate) struct ModulePath {
    pub(crate) parts: Vec<String>,
}

impl ModulePath {
    pub(crate) fn new(parts: Vec<String>) -> Self {
        Self { parts }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ImportItem {
    pub(crate) name: String,
    pub(crate) alias: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) enum Pattern {
    Wildcard,
    Binding(String),
    Literal(Literal),
    Or(Vec<Pattern>),
    Range {
        start: Box<Pattern>,
        end: Box<Pattern>,
        inclusive: bool,
    },
    Array(Vec<Pattern>),
    ArrayRest(Vec<Pattern>),
    Tuple(Vec<Pattern>),
    OptionSome(Box<Pattern>),
    ResultOk(Box<Pattern>),
    ResultErr(Box<Pattern>),
    EnumVariant {
        enum_name: String,
        variant_name: String,
    },
}

#[derive(Debug, Clone)]
pub(crate) struct MatchArm {
    pub(crate) pattern: Pattern,
    pub(crate) guard: Option<Expression>,
    pub(crate) body: Vec<Statement>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Visibility {
    Public,
    Protected,
    Private,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ClassField {
    pub(crate) name: String,
    pub(crate) visibility: Visibility,
    pub(crate) is_static: bool,
    pub(crate) type_annotation: Option<TypeExpr>,
    pub(crate) initializer: Option<Expression>,
    pub(crate) line: usize,
    pub(crate) column: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct FunctionMethod {
    pub(crate) name: String,
    pub(crate) generic_params: Vec<GenericParam>,
    pub(crate) visibility: Visibility,
    pub(crate) is_static: bool,
    pub(crate) params: Vec<String>,
    pub(crate) param_types: Vec<Option<TypeExpr>>,
    pub(crate) return_type: Option<TypeExpr>,
    pub(crate) body: Vec<Statement>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct InterfaceMethod {
    pub(crate) name: String,
    pub(crate) generic_params: Vec<GenericParam>,
    pub(crate) arity: usize,
    pub(crate) params: Vec<String>,
    pub(crate) param_types: Vec<Option<TypeExpr>>,
    pub(crate) return_type: Option<TypeExpr>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) enum Statement {
    Positioned {
        line: usize,
        column: usize,
        statement: Box<Statement>,
    },
    Let {
        name: String,
        value: Expression,
        mutable: bool,
        type_annotation: Option<TypeExpr>,
    },
    Assignment {
        target: AssignmentTarget,
        value: Expression,
    },
    Expression {
        expression: Expression,
    },
    Block(Vec<Statement>),
    If {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
    ForIn {
        variable: String,
        iterable: Expression,
        body: Vec<Statement>,
    },
    Match {
        value: Expression,
        arms: Vec<MatchArm>,
    },
    Throw {
        value: Expression,
    },
    Try {
        try_body: Vec<Statement>,
        catch_name: Option<String>,
        catch_type: Option<TypeExpr>,
        catch_body: Option<Vec<Statement>>,
        finally_body: Option<Vec<Statement>>,
    },
    Function {
        name: String,
        generic_params: Vec<GenericParam>,
        params: Vec<String>,
        param_types: Vec<Option<TypeExpr>>,
        return_type: Option<TypeExpr>,
        body: Vec<Statement>,
        is_async: bool,
    },
    Return {
        value: Option<Expression>,
    },
    Import {
        path: Vec<String>,
    },
    FromImport {
        module: ModulePath,
        items: Vec<ImportItem>,
    },
    Export {
        statement: Box<Statement>,
    },
    TypeAlias {
        name: String,
        generic_params: Vec<GenericParam>,
        type_expr: TypeExpr,
    },
    Class {
        name: String,
        generic_params: Vec<GenericParam>,
        bases: Vec<TypeExpr>,
        fields: Vec<ClassField>,
        methods: Vec<FunctionMethod>,
    },
    Enum {
        name: String,
        generic_params: Vec<GenericParam>,
        variants: Vec<String>,
        methods: Vec<FunctionMethod>,
    },
    Interface {
        name: String,
        generic_params: Vec<GenericParam>,
        bases: Vec<TypeExpr>,
        methods: Vec<InterfaceMethod>,
    },
    Break,
    Continue,
}

#[derive(Debug, Clone)]
pub(crate) enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    None,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) enum Expression {
    Literal(Literal),
    Variable(String),
    Unary {
        operator: UnaryOp,
        right: Box<Expression>,
        line: usize,
        column: usize,
    },
    Binary {
        left: Box<Expression>,
        operator: BinaryOp,
        right: Box<Expression>,
        line: usize,
        column: usize,
    },
    Function {
        params: Vec<String>,
        body: Vec<Statement>,
    },
    Call {
        callee: Box<Expression>,
        generic_args: Vec<TypeExpr>,
        arguments: Vec<Expression>,
        line: usize,
        column: usize,
    },
    Member {
        object: Box<Expression>,
        name: String,
        line: usize,
        column: usize,
    },
    Index {
        object: Box<Expression>,
        index: Box<Expression>,
        line: usize,
        column: usize,
    },
    New {
        class_name: String,
        generic_args: Vec<TypeExpr>,
        arguments: Vec<Expression>,
        line: usize,
        column: usize,
    },
    SelfValue,
    Array(Vec<Expression>),
    Tuple(Vec<Expression>),
    Dict(Vec<(String, Expression)>),
    Record(Vec<(String, Expression)>),
    Try(Box<Expression>),
    Await(Box<Expression>),
    Ternary {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum UnaryOp {
    Negate,
    Not,
    BitNot,
}

#[derive(Debug, Clone)]
pub(crate) enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    Is,
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
}

/// Modèle intermédiaire sémantique d'un module.
///
/// Contrairement à la v1, les nœuds sont possédés par la HIM : le backend ne
/// peut plus accéder à un nœud de l'AST par référence.
#[derive(Clone)]
pub(crate) struct HimModule {
    statements: Vec<Statement>,
    context: Option<TypeCheckContext>,
    resolved_calls: Rc<ResolvedCallTable>,
    resolved_members: Rc<ResolvedMemberTable>,
}

impl HimModule {
    pub(crate) fn statements(&self) -> &[Statement] {
        &self.statements
    }

    pub(crate) fn context(&self) -> Option<&TypeCheckContext> {
        self.context.as_ref()
    }

    pub(crate) fn resolved_calls(&self) -> Rc<ResolvedCallTable> {
        Rc::clone(&self.resolved_calls)
    }

    pub(crate) fn resolved_members(&self) -> Rc<ResolvedMemberTable> {
        Rc::clone(&self.resolved_members)
    }
}

pub(crate) struct HimBuilder;

impl HimBuilder {
    pub(crate) fn build(
        statements: &[ast::Statement],
        context: Option<TypeCheckContext>,
    ) -> Result<HimModule, CompileError> {
        let type_check = TypeChecker::check_for_compiler(statements, context.clone())?;

        Ok(HimModule {
            statements: statements.iter().map(lower_statement).collect(),
            context,
            resolved_calls: Rc::new(type_check.resolved_calls),
            resolved_members: Rc::new(type_check.resolved_members),
        })
    }
}

fn lower_statement(statement: &ast::Statement) -> Statement {
    match statement {
        ast::Statement::Positioned {
            line,
            column,
            statement,
        } => Statement::Positioned {
            line: *line,
            column: *column,
            statement: Box::new(lower_statement(statement)),
        },
        ast::Statement::Let {
            name,
            value,
            mutable,
            type_annotation,
        } => Statement::Let {
            name: name.clone(),
            value: lower_expression(value),
            mutable: *mutable,
            type_annotation: type_annotation.as_ref().map(lower_type_expr),
        },
        ast::Statement::Assignment { target, value } => Statement::Assignment {
            target: lower_assignment_target(target),
            value: lower_expression(value),
        },
        ast::Statement::Expression { expression } => Statement::Expression {
            expression: lower_expression(expression),
        },
        ast::Statement::Block(statements) => {
            Statement::Block(statements.iter().map(lower_statement).collect())
        }
        ast::Statement::If {
            condition,
            then_branch,
            else_branch,
        } => Statement::If {
            condition: lower_expression(condition),
            then_branch: then_branch.iter().map(lower_statement).collect(),
            else_branch: else_branch
                .as_ref()
                .map(|branch| branch.iter().map(lower_statement).collect()),
        },
        ast::Statement::While { condition, body } => Statement::While {
            condition: lower_expression(condition),
            body: body.iter().map(lower_statement).collect(),
        },
        ast::Statement::ForIn {
            variable,
            iterable,
            body,
        } => Statement::ForIn {
            variable: variable.clone(),
            iterable: lower_expression(iterable),
            body: body.iter().map(lower_statement).collect(),
        },
        ast::Statement::Match { value, arms } => Statement::Match {
            value: lower_expression(value),
            arms: arms.iter().map(lower_match_arm).collect(),
        },
        ast::Statement::Throw { value } => Statement::Throw {
            value: lower_expression(value),
        },
        ast::Statement::Try {
            try_body,
            catch_name,
            catch_type,
            catch_body,
            finally_body,
        } => Statement::Try {
            try_body: try_body.iter().map(lower_statement).collect(),
            catch_name: catch_name.clone(),
            catch_type: catch_type.as_ref().map(lower_type_expr),
            catch_body: catch_body
                .as_ref()
                .map(|body| body.iter().map(lower_statement).collect()),
            finally_body: finally_body
                .as_ref()
                .map(|body| body.iter().map(lower_statement).collect()),
        },
        ast::Statement::Function {
            name,
            generic_params,
            params,
            param_types,
            return_type,
            body,
            is_async,
        } => Statement::Function {
            name: name.clone(),
            generic_params: generic_params.iter().map(lower_generic_param).collect(),
            params: params.clone(),
            param_types: param_types
                .iter()
                .map(|type_expr| type_expr.as_ref().map(lower_type_expr))
                .collect(),
            return_type: return_type.as_ref().map(lower_type_expr),
            body: body.iter().map(lower_statement).collect(),
            is_async: *is_async,
        },
        ast::Statement::Return { value } => Statement::Return {
            value: value.as_ref().map(lower_expression),
        },
        ast::Statement::Import { path } => Statement::Import { path: path.clone() },
        ast::Statement::FromImport { module, items } => Statement::FromImport {
            module: ModulePath::new(module.parts.clone()),
            items: items.iter().map(lower_import_item).collect(),
        },
        ast::Statement::Export { statement } => Statement::Export {
            statement: Box::new(lower_statement(statement)),
        },
        ast::Statement::TypeAlias {
            name,
            generic_params,
            type_expr,
        } => Statement::TypeAlias {
            name: name.clone(),
            generic_params: generic_params.iter().map(lower_generic_param).collect(),
            type_expr: lower_type_expr(type_expr),
        },
        ast::Statement::Class {
            name,
            generic_params,
            bases,
            fields,
            methods,
        } => Statement::Class {
            name: name.clone(),
            generic_params: generic_params.iter().map(lower_generic_param).collect(),
            bases: bases.iter().map(lower_type_expr).collect(),
            fields: fields.iter().map(lower_class_field).collect(),
            methods: methods.iter().map(lower_function_method).collect(),
        },
        ast::Statement::Enum {
            name,
            generic_params,
            variants,
            methods,
        } => Statement::Enum {
            name: name.clone(),
            generic_params: generic_params.iter().map(lower_generic_param).collect(),
            variants: variants.clone(),
            methods: methods.iter().map(lower_function_method).collect(),
        },
        ast::Statement::Interface {
            name,
            generic_params,
            bases,
            methods,
        } => Statement::Interface {
            name: name.clone(),
            generic_params: generic_params.iter().map(lower_generic_param).collect(),
            bases: bases.iter().map(lower_type_expr).collect(),
            methods: methods.iter().map(lower_interface_method).collect(),
        },
        ast::Statement::Break => Statement::Break,
        ast::Statement::Continue => Statement::Continue,
    }
}

fn lower_generic_param(param: &ast::GenericParam) -> GenericParam {
    GenericParam {
        name: param.name.clone(),
        bounds: param.bounds.iter().map(lower_type_expr).collect(),
    }
}

fn lower_type_expr(type_expr: &ast::TypeExpr) -> TypeExpr {
    match type_expr {
        ast::TypeExpr::Named(name) => TypeExpr::Named(name.clone()),
        ast::TypeExpr::Generic { name, arguments } => TypeExpr::Generic {
            name: name.clone(),
            arguments: arguments.iter().map(lower_type_expr).collect(),
        },
        ast::TypeExpr::Union(types) => {
            TypeExpr::Union(types.iter().map(lower_type_expr).collect())
        }
        ast::TypeExpr::Record(fields) => TypeExpr::Record(
            fields
                .iter()
                .map(|(name, type_expr)| (name.clone(), lower_type_expr(type_expr)))
                .collect(),
        ),
    }
}

fn lower_assignment_target(target: &ast::AssignmentTarget) -> AssignmentTarget {
    match target {
        ast::AssignmentTarget::Variable(name) => AssignmentTarget::Variable(name.clone()),
        ast::AssignmentTarget::Index { object, index } => AssignmentTarget::Index {
            object: Box::new(lower_expression(object)),
            index: Box::new(lower_expression(index)),
        },
        ast::AssignmentTarget::Member { object, name } => AssignmentTarget::Member {
            object: Box::new(lower_expression(object)),
            name: name.clone(),
        },
    }
}

fn lower_match_arm(arm: &ast::MatchArm) -> MatchArm {
    MatchArm {
        pattern: lower_pattern(&arm.pattern),
        guard: arm.guard.as_ref().map(lower_expression),
        body: arm.body.iter().map(lower_statement).collect(),
    }
}

fn lower_pattern(pattern: &ast::Pattern) -> Pattern {
    match pattern {
        ast::Pattern::Wildcard => Pattern::Wildcard,
        ast::Pattern::Binding(name) => Pattern::Binding(name.clone()),
        ast::Pattern::Literal(literal) => Pattern::Literal(lower_literal(literal)),
        ast::Pattern::Or(patterns) => Pattern::Or(patterns.iter().map(lower_pattern).collect()),
        ast::Pattern::Range {
            start,
            end,
            inclusive,
        } => Pattern::Range {
            start: Box::new(lower_pattern(start)),
            end: Box::new(lower_pattern(end)),
            inclusive: *inclusive,
        },
        ast::Pattern::Array(patterns) => {
            Pattern::Array(patterns.iter().map(lower_pattern).collect())
        }
        ast::Pattern::ArrayRest(patterns) => {
            Pattern::ArrayRest(patterns.iter().map(lower_pattern).collect())
        }
        ast::Pattern::Tuple(patterns) => {
            Pattern::Tuple(patterns.iter().map(lower_pattern).collect())
        }
        ast::Pattern::OptionSome(pattern) => Pattern::OptionSome(Box::new(lower_pattern(pattern))),
        ast::Pattern::ResultOk(pattern) => Pattern::ResultOk(Box::new(lower_pattern(pattern))),
        ast::Pattern::ResultErr(pattern) => Pattern::ResultErr(Box::new(lower_pattern(pattern))),
        ast::Pattern::EnumVariant {
            enum_name,
            variant_name,
        } => Pattern::EnumVariant {
            enum_name: enum_name.clone(),
            variant_name: variant_name.clone(),
        },
    }
}

fn lower_class_field(field: &ast::ClassField) -> ClassField {
    ClassField {
        name: field.name.clone(),
        visibility: lower_visibility(field.visibility),
        is_static: field.is_static,
        type_annotation: field.type_annotation.as_ref().map(lower_type_expr),
        initializer: field.initializer.as_ref().map(lower_expression),
        line: field.line,
        column: field.column,
    }
}

fn lower_function_method(method: &ast::FunctionMethod) -> FunctionMethod {
    FunctionMethod {
        name: method.name.clone(),
        generic_params: method
            .generic_params
            .iter()
            .map(lower_generic_param)
            .collect(),
        visibility: lower_visibility(method.visibility),
        is_static: method.is_static,
        params: method.params.clone(),
        param_types: method
            .param_types
            .iter()
            .map(|type_expr| type_expr.as_ref().map(lower_type_expr))
            .collect(),
        return_type: method.return_type.as_ref().map(lower_type_expr),
        body: method.body.iter().map(lower_statement).collect(),
    }
}

fn lower_interface_method(method: &ast::InterfaceMethod) -> InterfaceMethod {
    InterfaceMethod {
        name: method.name.clone(),
        generic_params: method
            .generic_params
            .iter()
            .map(lower_generic_param)
            .collect(),
        arity: method.arity,
        params: method.params.clone(),
        param_types: method
            .param_types
            .iter()
            .map(|type_expr| type_expr.as_ref().map(lower_type_expr))
            .collect(),
        return_type: method.return_type.as_ref().map(lower_type_expr),
    }
}

fn lower_visibility(visibility: ast::Visibility) -> Visibility {
    match visibility {
        ast::Visibility::Public => Visibility::Public,
        ast::Visibility::Protected => Visibility::Protected,
        ast::Visibility::Private => Visibility::Private,
    }
}

fn lower_import_item(item: &ast::ImportItem) -> ImportItem {
    ImportItem {
        name: item.name.clone(),
        alias: item.alias.clone(),
    }
}

fn lower_literal(literal: &ast::Literal) -> Literal {
    match literal {
        ast::Literal::Integer(value) => Literal::Integer(*value),
        ast::Literal::Float(value) => Literal::Float(*value),
        ast::Literal::String(value) => Literal::String(value.clone()),
        ast::Literal::Bool(value) => Literal::Bool(*value),
        ast::Literal::None => Literal::None,
    }
}

fn lower_expression(expression: &ast::Expression) -> Expression {
    match expression {
        ast::Expression::Literal(literal) => Expression::Literal(lower_literal(literal)),
        ast::Expression::Variable(name) => Expression::Variable(name.clone()),
        ast::Expression::Unary {
            operator,
            right,
            line,
            column,
        } => Expression::Unary {
            operator: lower_unary_op(operator),
            right: Box::new(lower_expression(right)),
            line: *line,
            column: *column,
        },
        ast::Expression::Binary {
            left,
            operator,
            right,
            line,
            column,
        } => Expression::Binary {
            left: Box::new(lower_expression(left)),
            operator: lower_binary_op(operator),
            right: Box::new(lower_expression(right)),
            line: *line,
            column: *column,
        },
        ast::Expression::Function { params, body } => Expression::Function {
            params: params.clone(),
            body: body.iter().map(lower_statement).collect(),
        },
        ast::Expression::Call {
            callee,
            generic_args,
            arguments,
            line,
            column,
        } => Expression::Call {
            callee: Box::new(lower_expression(callee)),
            generic_args: generic_args.iter().map(lower_type_expr).collect(),
            arguments: arguments.iter().map(lower_expression).collect(),
            line: *line,
            column: *column,
        },
        ast::Expression::Member {
            object,
            name,
            line,
            column,
        } => Expression::Member {
            object: Box::new(lower_expression(object)),
            name: name.clone(),
            line: *line,
            column: *column,
        },
        ast::Expression::Index {
            object,
            index,
            line,
            column,
        } => Expression::Index {
            object: Box::new(lower_expression(object)),
            index: Box::new(lower_expression(index)),
            line: *line,
            column: *column,
        },
        ast::Expression::New {
            class_name,
            generic_args,
            arguments,
            line,
            column,
        } => Expression::New {
            class_name: class_name.clone(),
            generic_args: generic_args.iter().map(lower_type_expr).collect(),
            arguments: arguments.iter().map(lower_expression).collect(),
            line: *line,
            column: *column,
        },
        ast::Expression::SelfValue => Expression::SelfValue,
        ast::Expression::Array(elements) => {
            Expression::Array(elements.iter().map(lower_expression).collect())
        }
        ast::Expression::Tuple(elements) => {
            Expression::Tuple(elements.iter().map(lower_expression).collect())
        }
        ast::Expression::Dict(fields) => Expression::Dict(
            fields
                .iter()
                .map(|(name, value)| (name.clone(), lower_expression(value)))
                .collect(),
        ),
        ast::Expression::Record(fields) => Expression::Record(
            fields
                .iter()
                .map(|(name, value)| (name.clone(), lower_expression(value)))
                .collect(),
        ),
        ast::Expression::Try(expression) => Expression::Try(Box::new(lower_expression(expression))),
        ast::Expression::Await(expression) => {
            Expression::Await(Box::new(lower_expression(expression)))
        }
        ast::Expression::Ternary {
            condition,
            then_expr,
            else_expr,
        } => Expression::Ternary {
            condition: Box::new(lower_expression(condition)),
            then_expr: Box::new(lower_expression(then_expr)),
            else_expr: Box::new(lower_expression(else_expr)),
        },
    }
}

fn lower_unary_op(operator: &ast::UnaryOp) -> UnaryOp {
    match operator {
        ast::UnaryOp::Negate => UnaryOp::Negate,
        ast::UnaryOp::Not => UnaryOp::Not,
        ast::UnaryOp::BitNot => UnaryOp::BitNot,
    }
}

fn lower_binary_op(operator: &ast::BinaryOp) -> BinaryOp {
    match operator {
        ast::BinaryOp::Add => BinaryOp::Add,
        ast::BinaryOp::Subtract => BinaryOp::Subtract,
        ast::BinaryOp::Multiply => BinaryOp::Multiply,
        ast::BinaryOp::Divide => BinaryOp::Divide,
        ast::BinaryOp::Modulo => BinaryOp::Modulo,
        ast::BinaryOp::Equal => BinaryOp::Equal,
        ast::BinaryOp::NotEqual => BinaryOp::NotEqual,
        ast::BinaryOp::Less => BinaryOp::Less,
        ast::BinaryOp::LessEqual => BinaryOp::LessEqual,
        ast::BinaryOp::Greater => BinaryOp::Greater,
        ast::BinaryOp::GreaterEqual => BinaryOp::GreaterEqual,
        ast::BinaryOp::And => BinaryOp::And,
        ast::BinaryOp::Or => BinaryOp::Or,
        ast::BinaryOp::Is => BinaryOp::Is,
        ast::BinaryOp::BitAnd => BinaryOp::BitAnd,
        ast::BinaryOp::BitOr => BinaryOp::BitOr,
        ast::BinaryOp::BitXor => BinaryOp::BitXor,
        ast::BinaryOp::ShiftLeft => BinaryOp::ShiftLeft,
        ast::BinaryOp::ShiftRight => BinaryOp::ShiftRight,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::{lexer::lexer::Lexer, parser::Parser};

    fn parse(source: &str) -> Vec<ast::Statement> {
        let tokens = Lexer::new(source.to_string())
            .scan_token()
            .expect("lexing doit réussir");
        Parser::new(tokens).parse().expect("parsing doit réussir")
    }

    #[test]
    fn builds_from_valid_ast() {
        let statements = parse("let x: int = 1; x = x + 2;");
        let him = HimBuilder::build(&statements, None).expect("HIM valide attendue");

        assert_eq!(him.statements().len(), statements.len());
        assert_eq!(him.resolved_calls().iter().count(), 0);
    }

    #[test]
    fn owns_its_nodes_instead_of_borrowing_the_ast() {
        let statements = parse("let x = 1; println(x);");
        let him = HimBuilder::build(&statements, None).expect("HIM valide attendue");

        drop(statements);
        assert_eq!(him.statements().len(), 2);

        match &him.statements()[0] {
            Statement::Positioned { statement, .. } => {
                assert!(matches!(statement.as_ref(), Statement::Let { .. }));
            }
            Statement::Let { .. } => {}
            other => panic!("premier nœud HIM inattendu: {other:?}"),
        }
    }

    #[test]
    fn preserves_semantic_call_resolution() {
        let statements = parse("println(42);");
        let him = HimBuilder::build(&statements, None).expect("HIM valide attendue");

        assert!(him.resolved_calls().iter().count() > 0);
    }
}

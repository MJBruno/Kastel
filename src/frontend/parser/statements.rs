use crate::error::parse_error::ParserError;
use crate::frontend::ast::*;
use crate::frontend::lexer::token::TokenKind;

use super::Parser;

#[allow(dead_code)]
impl Parser {
    // ============================================================
    // STATEMENTS
    // ============================================================

    pub(super) fn statement(&mut self) -> Result<Vec<Statement>, ParserError> {
        let line = self.peek().line;
        let column = self.peek().column;

        let statements = if self.match_token(TokenKind::Import) {
            vec![self.parse_import_statement()?]
        } else if self.match_token(TokenKind::From) {
            vec![self.parse_from_import_statement()?]
        } else if self.match_token(TokenKind::Export) {
            vec![self.parse_export_statement()?]
        } else if self.match_token(TokenKind::Let) {
            self.parse_variable_declaration(true)?
        } else if self.match_token(TokenKind::Const) {
            self.parse_variable_declaration(false)?
        } else if self.match_token(TokenKind::Function) {
            vec![self.parse_function_statement()?]
        } else if self.match_token(TokenKind::Class) {
            vec![self.parse_class_statement()?]
        } else if self.match_token(TokenKind::Interface) {
            vec![self.parse_interface_statement()?]
        } else if self.match_token(TokenKind::Return) {
            vec![self.parse_return_statement()?]
        } else if self.match_token(TokenKind::Break) {
            vec![Statement::Break]
        } else if self.match_token(TokenKind::Continue) {
            vec![Statement::Continue]
        } else if self.match_token(TokenKind::Match) {
            vec![self.parse_match_statement()?]
        } else if self.match_token(TokenKind::Try) {
            vec![self.parse_try_statement()?]
        } else if self.match_token(TokenKind::Throw) {
            vec![self.parse_throw_statement()?]
        } else if self.match_token(TokenKind::If) {
            vec![self.parse_if_statement()?]
        } else if self.match_token(TokenKind::While) {
            vec![self.parse_while_statement()?]
        } else if self.match_token(TokenKind::For) {
            vec![self.parse_for_statement()?]
        } else if self.match_token(TokenKind::LeftBrace) {
            vec![Statement::Block(self.parse_block_statement()?)]
        } else {
            vec![self.parse_expression_or_assignment()?]
        };

        self.match_token(TokenKind::Semicolon);

        let positioned = statements
            .into_iter()
            .map(|statement| Statement::Positioned {
                line,
                column,
                statement: Box::new(statement),
            })
            .collect();

        Ok(positioned)
    }

    // ============================================================
    // EXPRESSION / ASSIGNMENT
    // ============================================================

    fn parse_expression_or_assignment(&mut self) -> Result<Statement, ParserError> {
        let expression = self.parse_expression()?;

        if self.match_token(TokenKind::Equal) {
            let value = self.parse_expression()?;

            let target = self.expression_to_assignment_target(expression)?;

            return Ok(Statement::Assignment { target, value });
        }

        if let Some(operator) = self.match_compound_assignment() {
            let value_expr = self.parse_expression()?;

            let target = self.expression_to_assignment_target(expression.clone())?;

            let left = Self::assignment_target_to_expression(&target);

            let value = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(value_expr),
            };

            return Ok(Statement::Assignment { target, value });
        }

        Ok(Statement::Expression { expression })
    }

    fn match_compound_assignment(&mut self) -> Option<BinaryOp> {
        let operator = match self.peek().kind {
            TokenKind::PlusEqual => BinaryOp::Add,
            TokenKind::MinusEqual => BinaryOp::Subtract,
            TokenKind::StarEqual => BinaryOp::Multiply,
            TokenKind::SlashEqual => BinaryOp::Divide,
            TokenKind::PercentEqual => BinaryOp::Modulo,
            _ => return None,
        };

        self.advance();

        Some(operator)
    }

    fn assignment_target_to_expression(target: &AssignmentTarget) -> Expression {
        match target {
            AssignmentTarget::Variable(name) => Expression::Variable(name.clone()),

            AssignmentTarget::Index { object, index } => Expression::Index {
                object: object.clone(),
                index: index.clone(),
            },

            AssignmentTarget::Member { object, name } => Expression::Member {
                object: object.clone(),
                name: name.clone(),
            },
        }
    }

    fn expression_to_assignment_target(
        &self,
        expression: Expression,
    ) -> Result<AssignmentTarget, ParserError> {
        match expression {
            Expression::Variable(name) => Ok(AssignmentTarget::Variable(name)),

            Expression::Index { object, index } => Ok(AssignmentTarget::Index { object, index }),

            Expression::Member { object, name } => Ok(AssignmentTarget::Member { object, name }),

            _ => {
                let token = self.peek();

                Err(ParserError {
                    message: "Cible d'affectation invalide".to_string(),
                    line: token.line,
                    column: token.column,
                })
            }
        }
    }

    // ============================================================
    // RETURN
    // ============================================================

    fn parse_return_statement(&mut self) -> Result<Statement, ParserError> {
        let value = if self.check(TokenKind::Semicolon)
            || self.check(TokenKind::RightBrace)
            || self.is_at_end()
        {
            None
        } else {
            Some(self.parse_expression()?)
        };

        Ok(Statement::Return { value })
    }

    // ============================================================
    // BLOCK
    // ============================================================

    pub(super) fn parse_block_statement(&mut self) -> Result<Vec<Statement>, ParserError> {
        let mut statements = Vec::new();

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            let parsed = self.statement()?;

            statements.extend(parsed);
        }

        self.consume(TokenKind::RightBrace, "'}' attendu après le bloc")?;

        Ok(statements)
    }

    // ============================================================
    // IF
    // ============================================================

    fn parse_if_statement(&mut self) -> Result<Statement, ParserError> {
        let condition = self.parse_expression()?;

        self.consume(TokenKind::LeftBrace, "'{' attendu après la condition")?;

        let then_branch = self.parse_block_statement()?;

        let else_branch = if self.match_token(TokenKind::Else) {
            self.consume(TokenKind::LeftBrace, "'{' attendu après else")?;

            Some(self.parse_block_statement()?)
        } else {
            None
        };

        Ok(Statement::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    // ============================================================
    // WHILE
    // ============================================================

    fn parse_while_statement(&mut self) -> Result<Statement, ParserError> {
        let condition = self.parse_expression()?;

        self.consume(TokenKind::LeftBrace, "'{' attendu après la condition")?;

        let body = self.parse_block_statement()?;

        Ok(Statement::While { condition, body })
    }

    // ============================================================
    // FOR..IN
    // ============================================================

    fn parse_for_statement(&mut self) -> Result<Statement, ParserError> {
        let variable =
            self.consume(TokenKind::Identifier, "Nom de variable attendu après 'for'")?;

        self.consume(TokenKind::In, "'in' attendu après le nom de variable")?;

        let iterable = self.parse_expression()?;

        self.consume(TokenKind::LeftBrace, "'{' attendu avant le corps du for")?;

        let body = self.parse_block_statement()?;

        Ok(Statement::ForIn {
            variable: variable.lexeme,
            iterable,
            body,
        })
    }
}
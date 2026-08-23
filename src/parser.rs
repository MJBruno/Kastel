use crate::ast::*;
use crate::token::{Token, TokenKind};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ParserError {
    pub message: String,

    pub line: usize,

    pub column: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    errors: Vec<ParserError>,
}
#[allow(dead_code)]
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            errors: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Statement>, Vec<ParserError>> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            match self.statement() {
                Ok(stmts) => {
                    statements.extend(stmts);
                }

                Err(error) => {
                    self.errors.push(error);
                    self.advance();
                }
            }
        }

        if self.errors.is_empty() {
            Ok(statements)
        } else {
            Err(self.errors.clone())
        }
    }

    // --------------------------------------------------
    //                  TOKEN HELPERS
    // --------------------------------------------------

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();

            true
        } else {
            false
        }
    }

    fn match_any(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(kind.clone()) {
                self.advance();

                return true;
            }
        }

        false
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    fn check_next(&self, kind: TokenKind) -> bool {
        if self.current + 1 >= self.tokens.len() {
            return false;
        }
        self.tokens[self.current + 1].kind == kind
    }

    fn consume(&mut self, kind: TokenKind, message: &str) -> Result<Token, ParserError> {
        if self.check(kind.clone()) {
            return Ok(self.advance().clone());
        }

        Err(ParserError {
            message: message.to_string(),

            line: self.peek().line,

            column: self.peek().column,
        })
    }

    fn parse_number(&self, token: Token) -> Result<Expression, ParserError> {
        let value = token
            .lexeme
            .parse::<f64>()
            .map_err(|_| format!("Invalid number '{}' at line {}", token.lexeme, token.line));
        Ok(Expression::Literal(Literal::Number(value.unwrap())))
    }

    fn parse_string(&self, token: Token) -> Result<Expression, ParserError> {
        let value = token.lexeme.trim_matches('"').to_string();
        Ok(Expression::Literal(Literal::String(value)))
    }

    // --------------------------------------------------
    //                  PARSE-EXPRESSION
    // --------------------------------------------------

    fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        let condition = self.logical_or()?;
        Ok(condition)
    }

    fn logical_or(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.logical_and()?;

        while self.match_token(TokenKind::Or) {
            let right = self.logical_and()?;

            expr = Expression::Binary {
                left: Box::new(expr),

                operator: BinaryOp::Or,

                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn logical_and(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.comparison()?;

        while self.match_token(TokenKind::And) {
            let right = self.comparison()?;

            expr = Expression::Binary {
                left: Box::new(expr),

                operator: BinaryOp::And,

                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.term()?;

        while self.match_any(&[
            TokenKind::EqualEqual,
            TokenKind::NotEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
            TokenKind::Greater,
            TokenKind::GreaterEqual,
        ]) {
            let operator = match self.previous().kind {
                TokenKind::EqualEqual => BinaryOp::Equal,
                TokenKind::NotEqual => BinaryOp::NotEqual,
                TokenKind::Less => BinaryOp::Less,
                TokenKind::LessEqual => BinaryOp::LessEqual,
                TokenKind::Greater => BinaryOp::Greater,
                TokenKind::GreaterEqual => BinaryOp::GreaterEqual,

                _ => unreachable!(),
            };

            let right = self.term()?;

            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }
    fn term(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.factor()?;

        while self.match_any(&[TokenKind::Plus, TokenKind::Minus]) {
            let operator = match self.previous().kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Subtract,

                _ => unreachable!(),
            };

            let right = self.factor()?;

            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.unary()?;

        while self.match_any(&[TokenKind::Star, TokenKind::Slash, TokenKind::Percent]) {
            let operator = match self.previous().kind {
                TokenKind::Star => BinaryOp::Multiply,
                TokenKind::Slash => BinaryOp::Divide,
                TokenKind::Percent => BinaryOp::Modulo,

                _ => unreachable!(),
            };

            let right = self.unary()?;

            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expression, ParserError> {
        if self.match_token(TokenKind::Minus) {
            let operand = self.unary()?;

            return Ok(Expression::Unary {
                operator: UnaryOp::Negate,
                right: Box::new(operand),
            });
        }

        if self.match_token(TokenKind::Not) {
            let operand = self.unary()?;

            return Ok(Expression::Unary {
                operator: UnaryOp::Not,
                right: Box::new(operand),
            });
        }

        self.call()
    }

    fn call(&mut self) -> Result<Expression, ParserError> {
        let mut expression = self.primary()?;

        loop {
            if self.match_token(TokenKind::LeftParen) {
                expression = self.parse_call(expression)?;
            } else {
                break;
            }
        }

        Ok(expression)
    }

    fn parse_call(&mut self, callee: Expression) -> Result<Expression, ParserError> {
        let mut arguments = Vec::new();
        if !self.check(TokenKind::RightParen) {
            loop {
                arguments.push(self.parse_expression()?);

                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RightParen, "message")?;

        Ok(Expression::Call {
            callee: Box::new(callee),
            arguments,
        })
    }

    fn primary(&mut self) -> Result<Expression, ParserError> {
        let token = self.advance().clone();

        match token.kind {
            TokenKind::Number => self.parse_number(token),
            TokenKind::String => self.parse_string(token),
            TokenKind::Identifier => Ok(Expression::Variable(token.lexeme)),
            TokenKind::True => Ok(Expression::Literal(Literal::Bool(true))),
            TokenKind::False => Ok(Expression::Literal(Literal::Bool(false))),
            TokenKind::Nil => Ok(Expression::Literal(Literal::Nil)),
            TokenKind::LeftParen => {
                let expr = self.parse_expression()?;

                self.consume(TokenKind::RightParen, "Parenthèse fermante attendue")?;

                Ok(expr)
            }

            _ => Err(ParserError {
                message: "Expression invalide".to_string(),
                line: token.line,
                column: token.column,
            }),
        }
    }

    fn statement(&mut self) -> Result<Vec<Statement>, ParserError> {
        let statements = if self.match_token(TokenKind::Let) {
            self.parse_let_statement()?
        } else if self.match_token(TokenKind::Function) {
            vec![self.parse_function_statement()?]
        } else if self.match_token(TokenKind::Return) {
            vec![self.parse_return_statement()?]
        } else if self.check(TokenKind::Identifier) && self.check_next(TokenKind::Equal) {
            vec![self.parse_assignment()?]
        } else if self.match_token(TokenKind::Print) {
            vec![self.parse_print_statement()?]
        } else if self.match_token(TokenKind::Break) {
            vec![Statement::Break]
        } else if self.match_token(TokenKind::Continue) {
            vec![Statement::Continue]
        } else if self.match_token(TokenKind::If) {
            vec![self.parse_if_statement()?]
        } else if self.match_token(TokenKind::While) {
            vec![self.parse_while_statement()?]
        } else if self.match_token(TokenKind::LeftBrace) {
            vec![Statement::Block(self.parse_block_statement()?)]
        } else {
            let expr = self.parse_expression()?;

            vec![Statement::Expression { expression: expr }]
        };

        // ; optionnel
        self.match_token(TokenKind::Semicolon);

        Ok(statements)
    }

    //////////////////////////
    // HELPER DE STATEMENTS //
    /////////////////////////

    fn parse_let_statement(&mut self) -> Result<Vec<Statement>, ParserError> {
        let mut declarations = Vec::new();

        loop {
            let name = self.consume(TokenKind::Identifier, "Nom de variable attendu")?;

            self.consume(TokenKind::Equal, " '=' attendu après le nom")?;

            let value = self.parse_expression()?;

            declarations.push(Statement::Let {
                name: name.lexeme,
                value,
            });

            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }

        Ok(declarations)
    }

    fn parse_function_statement(&mut self) -> Result<Statement, ParserError> {
        let name = self.consume(TokenKind::Identifier, "Nom de variable attendu")?;
        self.consume(TokenKind::LeftParen, " ( attendu après la condition")?;
        let mut params = Vec::new();
        if !self.check(TokenKind::RightParen) {
            loop {
                let param = self.consume(TokenKind::Identifier, "message")?;
                params.push(param.lexeme);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RightParen, "")?;
        self.consume(TokenKind::LeftBrace, "")?;

        let body = self.parse_block_statement()?;
        Ok(Statement::Function {
            name: name.lexeme,
            params,
            body,
        })
    }

    fn parse_print_statement(&mut self) -> Result<Statement, ParserError> {
        let expr = self.parse_expression()?;
        Ok(Statement::Print(expr))
    }

    fn parse_return_statement(&mut self) -> Result<Statement, ParserError> {
        let value = self.parse_expression()?;

        Ok(Statement::Return { value: Some(value) })
    }
    fn parse_assignment(&mut self) -> Result<Statement, ParserError> {
        let name = self.consume(TokenKind::Identifier, "Nom de variable attendu")?;
        self.consume(TokenKind::Equal, " = attendu")?;
        let value = self.parse_expression()?;
        Ok(Statement::Assignment {
            name: name.lexeme,
            value,
        })
    }

    fn consume_optional_semicolon(&mut self) {
        self.match_token(TokenKind::Semicolon);
    }

    fn parse_block_statement(&mut self) -> Result<Vec<Statement>, ParserError> {
        let mut statements = Vec::new();

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            let parsed = self.statement()?;
            statements.extend(parsed);
        }

        self.consume(TokenKind::RightBrace, " } attendu après le bloc")?;

        Ok(statements)
    }

    fn parse_if_statement(&mut self) -> Result<Statement, ParserError> {
        self.consume(TokenKind::LeftParen, " ( attendu après la condition")?;
        let condition = self.parse_expression()?;
        self.consume(TokenKind::RightParen, " ) attendu après la condition")?;
        self.consume(TokenKind::LeftBrace, " { attendu après la condition")?;
        let then_branch = self.parse_block_statement()?;
        let else_branch = if self.match_token(TokenKind::Else) {
            self.consume(TokenKind::LeftBrace, " { attendu après else")?;
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
    fn parse_while_statement(&mut self) -> Result<Statement, ParserError> {
        self.consume(TokenKind::LeftParen, " ( attendu après la condition")?;
        let condition = self.parse_expression()?;
        self.consume(TokenKind::RightParen, " ) attendu après la condition")?;
        self.consume(TokenKind::LeftBrace, " { attendu après la condition")?;
        let body = self.parse_block_statement()?;
        Ok(Statement::While { condition, body })
    }
}

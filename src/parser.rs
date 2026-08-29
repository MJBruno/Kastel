use crate::ast::*;
use crate::error::ParserError;
use crate::token::{Token, TokenKind};

#[derive(Debug, Clone)]
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

    // ============================================================
    // TOKEN HELPERS
    // ============================================================

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
        let value = token.lexeme.parse::<f64>().map_err(|_| ParserError {
            message: format!("Nombre invalide '{}'", token.lexeme),
            line: token.line,
            column: token.column,
        })?;

        Ok(Expression::Literal(Literal::Number(value)))
    }

    fn parse_string(&self, token: Token) -> Result<Expression, ParserError> {
        let value = token.lexeme.trim_matches('"').to_string();

        Ok(Expression::Literal(Literal::String(value)))
    }

    // ============================================================
    // EXPRESSIONS
    // ============================================================

    fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        self.logical_or()
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

    // ============================================================
    // CALL / INDEX
    // ============================================================

    fn call(&mut self) -> Result<Expression, ParserError> {
        let mut expression = self.primary()?;

        loop {
            // --------------------------------------------------------
            // Appel de fonction
            //
            // foo(...)
            // a.push(...)
            // a[0](...)
            // --------------------------------------------------------
            if self.match_token(TokenKind::LeftParen) {
                expression = self.parse_call(expression)?;

                continue;
            }

            // --------------------------------------------------------
            // Indexation
            //
            // a[0]
            // matrix[0][1]
            // matrix[0][1][2]
            // --------------------------------------------------------
            if self.match_token(TokenKind::LeftBracket) {
                let index = self.parse_expression()?;

                self.consume(TokenKind::RightBracket, "']' attendu après l'index")?;

                expression = Expression::Index {
                    object: Box::new(expression),
                    index: Box::new(index),
                };

                continue;
            }

            // --------------------------------------------------------
            // Accès membre
            //
            // a.length
            // a.push
            // a.pop
            // matrix[0].length
            // matrix[0].push
            // --------------------------------------------------------
            if self.match_token(TokenKind::Dot) {
                let name =
                    self.consume(TokenKind::Identifier, "Nom de membre attendu après '.'")?;

                expression = Expression::Member {
                    object: Box::new(expression),
                    name: name.lexeme,
                };

                continue;
            }

            break;
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

        self.consume(TokenKind::RightParen, "')' attendu après les arguments")?;

        Ok(Expression::Call {
            callee: Box::new(callee),
            arguments,
        })
    }

    // ============================================================
    // PRIMARY
    // ============================================================

    fn primary(&mut self) -> Result<Expression, ParserError> {
        let token = self.advance().clone();

        match token.kind {
            TokenKind::Number => self.parse_number(token),

            TokenKind::String => self.parse_string(token),

            TokenKind::Identifier => Ok(Expression::Variable(token.lexeme)),

            TokenKind::True => Ok(Expression::Literal(Literal::Bool(true))),

            TokenKind::False => Ok(Expression::Literal(Literal::Bool(false))),

            TokenKind::Nil => Ok(Expression::Literal(Literal::Nil)),

            // ----------------------------------------------------
            // Parenthèses
            // ----------------------------------------------------
            TokenKind::LeftParen => {
                let expression = self.parse_expression()?;

                self.consume(TokenKind::RightParen, "')' attendu après l'expression")?;

                Ok(expression)
            }

            // ----------------------------------------------------
            // Tableau
            // ----------------------------------------------------
            TokenKind::LeftBracket => {
                let mut elements = Vec::new();

                if !self.check(TokenKind::RightBracket) {
                    loop {
                        elements.push(self.parse_expression()?);

                        if !self.match_token(TokenKind::Comma) {
                            break;
                        }

                        // Autorise :
                        //
                        // [1, 2, 3,]
                        if self.check(TokenKind::RightBracket) {
                            break;
                        }
                    }
                }

                self.consume(TokenKind::RightBracket, "']' attendu après le tableau")?;

                Ok(Expression::Array(elements))
            }

            _ => Err(ParserError {
                message: "Expression invalide".to_string(),
                line: token.line,
                column: token.column,
            }),
        }
    }

    // ============================================================
    // STATEMENTS
    // ============================================================

    fn statement(&mut self) -> Result<Vec<Statement>, ParserError> {
        let statements = if self.match_token(TokenKind::Let) {
            self.parse_variable_declaration(true)?
        } else if self.match_token(TokenKind::Const) {
            self.parse_variable_declaration(false)?
        } else if self.match_token(TokenKind::Function) {
            vec![self.parse_function_statement()?]
        } else if self.match_token(TokenKind::Return) {
            vec![self.parse_return_statement()?]
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
            vec![self.parse_expression_or_assignment()?]
        };

        // ';' optionnel
        self.match_token(TokenKind::Semicolon);

        Ok(statements)
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

        Ok(Statement::Expression { expression })
    }

    fn expression_to_assignment_target(
        &self,
        expression: Expression,
    ) -> Result<AssignmentTarget, ParserError> {
        match expression {
            Expression::Variable(name) => Ok(AssignmentTarget::Variable(name)),

            Expression::Index { object, index } => Ok(AssignmentTarget::Index { object, index }),

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
    // DECLARATION
    // ============================================================

    fn parse_variable_declaration(&mut self, mutable: bool) -> Result<Vec<Statement>, ParserError> {
        let mut declarations = Vec::new();

        loop {
            let name = self.consume(TokenKind::Identifier, "Nom de variable attendu")?;

            self.consume(TokenKind::Equal, "'=' attendu après le nom")?;

            let value = self.parse_expression()?;

            declarations.push(Statement::Let {
                name: name.lexeme,
                value,
                mutable,
            });

            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }

        Ok(declarations)
    }

    // ============================================================
    // FUNCTION
    // ============================================================

    fn parse_function_statement(&mut self) -> Result<Statement, ParserError> {
        let name = self.consume(TokenKind::Identifier, "Nom de fonction attendu")?;

        self.consume(TokenKind::LeftParen, "'(' attendu après le nom de fonction")?;

        let mut params = Vec::new();

        if !self.check(TokenKind::RightParen) {
            loop {
                let param = self.consume(TokenKind::Identifier, "Nom de paramètre attendu")?;

                params.push(param.lexeme);

                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RightParen, "')' attendu après les paramètres")?;

        self.consume(TokenKind::LeftBrace, "'{' attendu avant le corps")?;

        let body = self.parse_block_statement()?;

        Ok(Statement::Function {
            name: name.lexeme,
            params,
            body,
        })
    }

    // ============================================================
    // PRINT
    // ============================================================

    fn parse_print_statement(&mut self) -> Result<Statement, ParserError> {
        let expression = self.parse_expression()?;

        Ok(Statement::Print(expression))
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

    fn parse_block_statement(&mut self) -> Result<Vec<Statement>, ParserError> {
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
        self.consume(TokenKind::LeftParen, "'(' attendu après if")?;

        let condition = self.parse_expression()?;

        self.consume(TokenKind::RightParen, "')' attendu après la condition")?;

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
        self.consume(TokenKind::LeftParen, "'(' attendu après while")?;

        let condition = self.parse_expression()?;

        self.consume(TokenKind::RightParen, "')' attendu après la condition")?;

        self.consume(TokenKind::LeftBrace, "'{' attendu après la condition")?;

        let body = self.parse_block_statement()?;

        Ok(Statement::While { condition, body })
    }
}

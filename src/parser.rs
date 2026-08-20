use crate::{
    ast::*,
    token::{Token, TokenKind},
};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
// #[allow(dead_code)]
enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}
// #[allow(dead_code)]
impl Precedence {
    fn next(self) -> Self {
        match self {
            Self::None => Self::Assignment,
            Self::Assignment => Self::Or,
            Self::Or => Self::And,
            Self::And => Self::Equality,
            Self::Equality => Self::Comparison,
            Self::Comparison => Self::Term,
            Self::Term => Self::Factor,
            Self::Factor => Self::Unary,
            Self::Unary => Self::Call,
            Self::Call => Self::Primary,
            Self::Primary => Self::Primary,
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }
    pub fn parse(&mut self) -> Result<Vec<Statement>, String> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            match self.statement() {
                Ok(stmt) => {
                    statements.push(stmt);
                }

                Err(error) => {
                    println!("{}", error);
                    self.advance();
                }
            }
        }
        Ok(statements)
    }

    pub fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_precedence(Precedence::Assignment)
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> Result<Expression, String> {
        let token = self.advance();

        let mut left = self.parse_prefix(token)?;

        while precedence <= self.current_precedence() {
            let token = self.advance();

            left = self.parse_infix(token, left)?;
        }

        Ok(left)
    }

    // --------------------------------------------------
    //                      PREFIX
    // --------------------------------------------------

    fn parse_prefix(&mut self, token: Token) -> Result<Expression, String> {
        match token.kind {
            TokenKind::Number => self.parse_number(token),
            TokenKind::String => self.parse_string(token),
            TokenKind::Identifier => Ok(Expression::Variable(token.lexeme)),
            TokenKind::True => Ok(Expression::Literal(Literal::Bool(true))),
            TokenKind::False => Ok(Expression::Literal(Literal::Bool(false))),
            TokenKind::Nil => Ok(Expression::Literal(Literal::Nil)),
            TokenKind::Minus => {
                let right = self.parse_precedence(Precedence::Unary)?;
                Ok(Expression::Unary {
                    operator: UnaryOp::Negate,
                    right: Box::new(right),
                })
            }

            TokenKind::Not => {
                let right = self.parse_precedence(Precedence::Unary)?;
                Ok(Expression::Unary {
                    operator: UnaryOp::Not,
                    right: Box::new(right),
                })
            }

            TokenKind::LeftParen => {
                let expr = self.parse_precedence(Precedence::Assignment)?;
                self.consume(TokenKind::RightParen, "Expected ')' after expression")?;
                Ok(expr)
            }

            _ => Err(format!("Unexpected token {:?}", token.kind)),
        }
    }

    // --------------------------------------------------
    //                      INFIX / POSTFIX
    // --------------------------------------------------

    fn parse_infix(&mut self, token: Token, left: Expression) -> Result<Expression, String> {
        match token.kind {
            TokenKind::Plus => self.binary(left, BinaryOp::Add, Precedence::Term),
            TokenKind::Minus => self.binary(left, BinaryOp::Subtract, Precedence::Term),
            TokenKind::Star => self.binary(left, BinaryOp::Multiply, Precedence::Factor),
            TokenKind::Slash => self.binary(left, BinaryOp::Divide, Precedence::Factor),
            TokenKind::Percent => self.binary(left, BinaryOp::Modulo, Precedence::Factor),
            TokenKind::EqualEqual => self.binary(left, BinaryOp::Equal, Precedence::Equality),
            TokenKind::NotEqual => self.binary(left, BinaryOp::NotEqual, Precedence::Equality),
            TokenKind::Less => self.binary(left, BinaryOp::Less, Precedence::Comparison),
            TokenKind::LessEqual => self.binary(left, BinaryOp::LessEqual, Precedence::Comparison),
            TokenKind::Greater => self.binary(left, BinaryOp::Greater, Precedence::Comparison),
            TokenKind::GreaterEqual => {
                self.binary(left, BinaryOp::GreaterEqual, Precedence::Comparison)
            }
            TokenKind::And => self.binary(left, BinaryOp::And, Precedence::And),
            TokenKind::Or => self.binary(left, BinaryOp::Or, Precedence::Or),
            TokenKind::Equal => self.assignment(left),
            _ => Err(format!("Invalid infix token {:?}", token.kind)),
        }
    }

    fn binary(
        &mut self,
        left: Expression,
        operator: BinaryOp,
        precedence: Precedence,
    ) -> Result<Expression, String> {
        let right = self.parse_precedence(precedence.next())?;

        Ok(Expression::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }

    fn assignment(&mut self, left: Expression) -> Result<Expression, String> {
        let value = self.parse_precedence(Precedence::Assignment)?;
        match left {
            Expression::Variable(name) => Ok(Expression::Assignment {
                name,
                value: Box::new(value),
            }),

            _ => Err("Invalid assignment target".to_string()),
        }
    }

    // --------------------------------------------------
    //                  PRECEDENCE
    // --------------------------------------------------

    fn current_precedence(&self) -> Precedence {
        match &self.peek().kind {
            TokenKind::Equal => Precedence::Assignment,

            TokenKind::Or => Precedence::Or,
            TokenKind::And => Precedence::And,

            TokenKind::EqualEqual | TokenKind::NotEqual => Precedence::Equality,

            TokenKind::Less
            | TokenKind::LessEqual
            | TokenKind::Greater
            | TokenKind::GreaterEqual => Precedence::Comparison,

            TokenKind::Plus | TokenKind::Minus => Precedence::Term,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Precedence::Factor,
            TokenKind::LeftParen | TokenKind::LeftBracket => Precedence::Call,

            _ => Precedence::None,
        }
    }

    fn statement(&mut self) -> Result<Statement, String> {
        //Déclaration variable let
        if self.match_token(TokenKind::Let) {
            return self.parse_let_statement();
        }

        if self.check(&TokenKind::Identifier) && self.check_next(TokenKind::Equal) {
            return self.parse_assignment();
        }

        if self.match_token(TokenKind::Print) {
            return self.parse_print_statement();
        }
        if self.match_token(TokenKind::Break) {
            return Ok(Statement::Break);
        }
        if self.match_token(TokenKind::Continue) {
            return Ok(Statement::Continue);
        }

        if self.match_token(TokenKind::If) {
            return self.parse_if_statement();
        }

        if self.match_token(TokenKind::While) {
            return self.parse_while_statement();
        }

        if self.match_token(TokenKind::LeftBrace) {
            let statements = self.parse_block_statement()?;
            return Ok(Statement::Block(statements));
        }

        let expr = self.parse_expression()?;

        Ok(Statement::Expression { expression: expr })
    }

    //////////////////////////
    // HELPER DE STATEMENTS //
    /////////////////////////

    fn parse_let_statement(&mut self) -> Result<Statement, String> {
        let name = self.consume(TokenKind::Identifier, "Nom de variable attendu")?;
        self.consume(TokenKind::Equal, " '=' attendu après le nom")?;
        let value = self.parse_expression()?;
        Ok(Statement::Let {
            name: name.lexeme,
            value,
        })
    }

    fn parse_assignment(&mut self) -> Result<Statement, String> {
        let name = self.consume(TokenKind::Identifier, "Nom de variable attendu")?;
        self.consume(TokenKind::Equal, " = attendu")?;
        let value = self.parse_expression()?;
        Ok(Statement::Assignment {
            name: name.lexeme,
            value,
        })
    }

    fn parse_block_statement(&mut self) -> Result<Vec<Statement>, String> {
        let mut statements = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.statement()?);
        }
        self.consume(TokenKind::RightBrace, " } attendu après le bloc")?;
        Ok(statements)
    }
    fn parse_print_statement(&mut self) -> Result<Statement, String> {
        let expr = self.parse_expression()?;
        Ok(Statement::Print(expr))
    }

    fn parse_if_statement(&mut self) -> Result<Statement, String> {
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
    fn parse_while_statement(&mut self) -> Result<Statement, String> {
        self.consume(TokenKind::LeftParen, " ( attendu après la condition")?;
        let condition = self.parse_expression()?;
        self.consume(TokenKind::RightParen, " ) attendu après la condition")?;
        self.consume(TokenKind::LeftBrace, " { attendu après la condition")?;
        let body = self.parse_block_statement()?;
        Ok(Statement::While { condition, body })
    }

    // --------------------------------------------------
    //                  TOKEN HELPERS
    // --------------------------------------------------

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.current].clone();

        if !self.check(&TokenKind::Eof) {
            self.current += 1;
        }

        token
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
    }

    fn check_next(&self, kind: TokenKind) -> bool {
        if self.current + 1 >= self.tokens.len() {
            return false;
        }
        self.tokens[self.current + 1].kind == kind
    }

    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(&kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    fn consume(&mut self, kind: TokenKind, message: &str) -> Result<Token, String> {
        if self.check(&kind) {
            return Ok(self.advance().clone());
        }
        Err(format!(
            "{} : {} : {}",
            message,
            self.peek().line,
            self.peek().column,
        ))
    }

    fn parse_number(&self, token: Token) -> Result<Expression, String> {
        let value = token
            .lexeme
            .parse::<f64>()
            .map_err(|_| format!("Invalid number '{}' at line {}", token.lexeme, token.line))?;
        Ok(Expression::Literal(Literal::Number(value)))
    }

    fn parse_string(&self, token: Token) -> Result<Expression, String> {
        let value = token.lexeme.trim_matches('"').to_string();
        Ok(Expression::Literal(Literal::String(value)))
    }
}

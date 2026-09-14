mod classes;
mod declarations;
mod exceptions;
mod expressions;
mod functions;
mod imports;
mod patterns;
mod statements;

use crate::error::parse_error::ParserError;
use crate::frontend::ast::*;
use crate::frontend::token::*;

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
        let is_float = token.lexeme.contains('.') || token.lexeme.contains(['e', 'E']);

        if is_float {
            let value = token.lexeme.parse::<f64>().map_err(|_| ParserError {
                message: format!("Nombre flottant invalide '{}'", token.lexeme),
                line: token.line,
                column: token.column,
            })?;

            Ok(Expression::Literal(Literal::Float(value)))
        } else {
            let value = token.lexeme.parse::<i64>().map_err(|_| ParserError {
                message: format!("Nombre entier invalide '{}'", token.lexeme),
                line: token.line,
                column: token.column,
            })?;

            Ok(Expression::Literal(Literal::Integer(value)))
        }
    }

    fn parse_string(&self, token: Token) -> Result<Expression, ParserError> {
        let value = token.lexeme.trim_matches('"').to_string();

        Ok(Expression::Literal(Literal::String(value)))
    }
}
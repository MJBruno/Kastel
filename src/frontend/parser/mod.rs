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
use crate::frontend::lexer::token::{Token, TokenKind};

/// Imbrication maximale (parenthèses, blocs, opérateurs unaires, fonctions
/// anonymes...). Le parser est récursif : sans limite, un source pathologique
/// (`((((...` sur des milliers de niveaux) ferait déborder la pile native.
pub const MAX_NESTING_DEPTH: usize = 500;

#[derive(Debug, Clone)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    errors: Vec<ParserError>,

    /// Profondeur d'imbrication courante (voir `MAX_NESTING_DEPTH`).
    depth: usize,
}

#[allow(dead_code)]
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            errors: Vec::new(),
            depth: 0,
        }
    }

    /// Exécute `f` un niveau plus profond ; refuse au-delà de
    /// `MAX_NESTING_DEPTH`. Le compteur est toujours rétabli, même en cas
    /// d'erreur.
    fn nested<T>(
        &mut self,
        f: impl FnOnce(&mut Self) -> Result<T, ParserError>,
    ) -> Result<T, ParserError> {
        if self.depth >= MAX_NESTING_DEPTH {
            let token = self.peek();

            return Err(ParserError {
                message: format!(
                    "Imbrication trop profonde (limite : {MAX_NESTING_DEPTH} niveaux)"
                ),
                line: token.line,
                column: token.column,
            });
        }

        self.depth += 1;
        let result = f(self);
        self.depth -= 1;

        result
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

    /// Erreur pour un littéral entier qui ne se lit pas comme `i64` :
    /// trop grand (chiffres seuls) ou mal formé.
    fn integer_literal_error(token: &Token) -> ParserError {
        let only_digits =
            !token.lexeme.is_empty() && token.lexeme.bytes().all(|b| b.is_ascii_digit());

        let message = if only_digits {
            format!(
                "Entier trop grand : {} dépasse la limite des entiers 64 bits (maximum {}). \
                 Écrivez-le en flottant (ex. {}.0) si la précision n'est pas essentielle.",
                token.lexeme,
                i64::MAX,
                token.lexeme
            )
        } else {
            format!("Nombre entier invalide '{}'", token.lexeme)
        };

        ParserError {
            message,
            line: token.line,
            column: token.column,
        }
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
            let value = token
                .lexeme
                .parse::<i64>()
                .map_err(|_| Self::integer_literal_error(&token))?;

            Ok(Expression::Literal(Literal::Integer(value)))
        }
    }

    fn parse_string(&self, token: Token) -> Result<Expression, ParserError> {
        Ok(Expression::Literal(Literal::String(token.lexeme)))
    }
}

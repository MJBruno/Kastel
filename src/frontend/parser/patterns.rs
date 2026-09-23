use crate::error::parse_error::ParserError;
use crate::frontend::ast::*;
use crate::frontend::lexer::token::TokenKind;

use super::Parser;

#[allow(dead_code)]
impl Parser {
    // ============================================================
    // MATCH
    // ============================================================

    pub(super) fn parse_match_statement(&mut self) -> Result<Statement, ParserError> {
        let value = self.parse_expression()?;

        self.consume(TokenKind::LeftBrace, "'{' attendu après l'expression match")?;

        let mut arms = Vec::new();

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            let pattern = self.parse_pattern()?;

            let guard = if self.match_token(TokenKind::If) {
                Some(self.parse_expression()?)
            } else {
                None
            };

            self.consume(TokenKind::FatArrow, "'=>' attendu après le pattern")?;

            let body = if self.match_token(TokenKind::LeftBrace) {
                // Forme bloc :
                // `pattern => { ... }`
                self.parse_block_statement()?
            } else {
                // Forme statement simple :
                // `pattern => return value;`
                //
                // `statement()` permet aussi de conserver les expressions
                // (`pattern => foo();`) sous forme de Statement::Expression,
                // tout en acceptant `return`, `let`, `throw`, etc.
                self.statement()?
            };

            arms.push(MatchArm {
                pattern,
                guard,
                body,
            });

            self.match_token(TokenKind::Comma);
        }

        self.consume(TokenKind::RightBrace, "'}' attendu après les arms du match")?;

        if arms.is_empty() {
            return Err(ParserError {
                message: "Un match doit contenir au moins un arm".to_string(),
                line: self.previous().line,
                column: self.previous().column,
            });
        }

        Ok(Statement::Match { value, arms })
    }

    fn parse_pattern(&mut self) -> Result<Pattern, ParserError> {
        let mut patterns = vec![self.parse_pattern_atom()?];

        while self.match_token(TokenKind::Pipe) {
            patterns.push(self.parse_pattern_atom()?);
        }

        if patterns.len() == 1 {
            Ok(patterns.remove(0))
        } else {
            Ok(Pattern::Or(patterns))
        }
    }

    fn parse_pattern_atom(&mut self) -> Result<Pattern, ParserError> {
        let token = self.peek().clone();

        let mut pattern = match token.kind {
            // ========================================================
            // WILDCARD / BINDING
            // ========================================================
            TokenKind::Identifier => {
                let token = self.advance().clone();

                if token.lexeme == "_" {
                    Pattern::Wildcard
                } else {
                    Pattern::Binding(token.lexeme)
                }
            }

            // ========================================================
            // LITTÉRAUX
            // ========================================================
            TokenKind::Number => {
                let token = self.advance().clone();

                let is_float = token.lexeme.contains('.') || token.lexeme.contains(['e', 'E']);

                if is_float {
                    let value = token.lexeme.parse::<f64>().map_err(|_| ParserError {
                        message: format!("Nombre flottant invalide '{}'", token.lexeme),
                        line: token.line,
                        column: token.column,
                    })?;

                    Pattern::Literal(Literal::Float(value))
                } else {
                    let value = token
                        .lexeme
                        .parse::<i64>()
                        .map_err(|_| Self::integer_literal_error(&token))?;

                    Pattern::Literal(Literal::Integer(value))
                }
            }

            TokenKind::String => {
                let token = self.advance().clone();

                Pattern::Literal(Literal::String(token.lexeme))
            }

            TokenKind::True => {
                self.advance();

                Pattern::Literal(Literal::Bool(true))
            }

            TokenKind::False => {
                self.advance();

                Pattern::Literal(Literal::Bool(false))
            }

            TokenKind::None => {
                self.advance();

                Pattern::Literal(Literal::None)
            }

            // ========================================================
            // TABLEAU
            // ========================================================
            TokenKind::LeftBracket => {
                self.advance();

                let mut patterns = Vec::new();

                if !self.check(TokenKind::RightBracket) {
                    loop {
                        patterns.push(self.parse_pattern()?);

                        if !self.match_token(TokenKind::Comma) {
                            break;
                        }

                        if self.check(TokenKind::RightBracket) {
                            break;
                        }
                    }
                }

                self.consume(
                    TokenKind::RightBracket,
                    "']' attendu après le pattern tableau",
                )?;

                Pattern::Array(patterns)
            }

            // ========================================================
            // PARENTHESES
            // ========================================================
            TokenKind::LeftParen => {
                self.advance();

                let pattern = self.parse_pattern()?;

                self.consume(TokenKind::RightParen, "')' attendu après le pattern")?;

                pattern
            }

            // ========================================================
            // NOMBRE NÉGATIF
            // ========================================================
            TokenKind::Minus => {
                self.advance();

                let token = self.consume(
                    TokenKind::Number,
                    "Nombre attendu après '-' dans un pattern",
                )?;

                let is_float = token.lexeme.contains('.') || token.lexeme.contains(['e', 'E']);

                if is_float {
                    let value = token.lexeme.parse::<f64>().map_err(|_| ParserError {
                        message: format!("Nombre flottant invalide '-{}'", token.lexeme),
                        line: token.line,
                        column: token.column,
                    })?;

                    Pattern::Literal(Literal::Float(-value))
                } else {
                    // On lit le littéral AVEC son signe : `-9223372036854775808`
                    // (i64::MIN) est valide alors que sa valeur absolue ne
                    // tient pas dans un i64.
                    let value = format!("-{}", token.lexeme)
                        .parse::<i64>()
                        .map_err(|_| Self::integer_literal_error(&token))?;

                    Pattern::Literal(Literal::Integer(value))
                }
            }

            _ => {
                return Err(ParserError {
                    message: "Pattern invalide".to_string(),
                    line: token.line,
                    column: token.column,
                });
            }
        };

        // ============================================================
        // RANGE
        // ============================================================

        if self.match_token(TokenKind::Range) {
            let end = self.parse_pattern_atom()?;

            pattern = Pattern::Range {
                start: Box::new(pattern),
                end: Box::new(end),
                inclusive: false,
            };
        } else if self.match_token(TokenKind::RangeInclusive) {
            let end = self.parse_pattern_atom()?;

            pattern = Pattern::Range {
                start: Box::new(pattern),
                end: Box::new(end),
                inclusive: true,
            };
        }

        Ok(pattern)
    }
}

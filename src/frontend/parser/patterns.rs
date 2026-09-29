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
                self.parse_block_statement()?
            } else {
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

    /// Parse un pattern complet. `|` reste réservé au OR-pattern, alors que
    /// les opérateurs logiques appartiennent aux expressions et ne sont donc
    /// jamais consommés ici.
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
            // WILDCARD / BINDING / CONSTRUCTEURS SPÉCIAUX
            // ========================================================
            TokenKind::Identifier => {
                let identifier = self.advance().clone();

                if identifier.lexeme == "_" {
                    Pattern::Wildcard
                } else if self.match_token(TokenKind::LeftParen) {
                    let argument = self.parse_pattern()?;
                    self.consume(
                        TokenKind::RightParen,
                        "')' attendu après le pattern constructeur",
                    )?;

                    match identifier.lexeme.as_str() {
                        "Some" => Pattern::OptionSome(Box::new(argument)),
                        "Ok" => Pattern::ResultOk(Box::new(argument)),
                        "Err" => Pattern::ResultErr(Box::new(argument)),
                        _ => {
                            return Err(ParserError {
                                message: format!(
                                    "Constructeur de pattern inconnu '{}'; utilisez Some(...), Ok(...) ou Err(...)",
                                    identifier.lexeme
                                ),
                                line: identifier.line,
                                column: identifier.column,
                            });
                        }
                    }
                } else if self.match_token(TokenKind::Dot) {
                    let variant = self.consume(
                        TokenKind::Identifier,
                        "Nom de variant attendu après '.' dans un pattern d'enum",
                    )?;

                    Pattern::EnumVariant {
                        enum_name: identifier.lexeme,
                        variant_name: variant.lexeme,
                    }
                } else {
                    Pattern::Binding(identifier.lexeme)
                }
            }

            // ========================================================
            // LITTÉRAUX
            // ========================================================
            TokenKind::Number => {
                let token = self.advance().clone();
                Self::number_pattern(token)?
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
            // TABLEAU / SLICE-PATTERN
            // ========================================================
            TokenKind::LeftBracket => self.parse_array_pattern()?,

            // ========================================================
            // TUPLE / GROUPEMENT
            // ========================================================
            TokenKind::LeftParen => self.parse_parenthesized_pattern()?,

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

    fn number_pattern(token: crate::frontend::lexer::token::Token) -> Result<Pattern, ParserError> {
        let is_float = token.lexeme.contains('.') || token.lexeme.contains(['e', 'E']);

        if is_float {
            let value = token.lexeme.parse::<f64>().map_err(|_| ParserError {
                message: format!("Nombre flottant invalide '{}'", token.lexeme),
                line: token.line,
                column: token.column,
            })?;

            Ok(Pattern::Literal(Literal::Float(value)))
        } else {
            let value = token
                .lexeme
                .parse::<i64>()
                .map_err(|_| Self::integer_literal_error(&token))?;

            Ok(Pattern::Literal(Literal::Integer(value)))
        }
    }

    fn parse_parenthesized_pattern(&mut self) -> Result<Pattern, ParserError> {
        self.advance(); // '('

        if self.check(TokenKind::RightParen) {
            self.advance();
            return Ok(Pattern::Tuple(Vec::new()));
        }

        let first = self.parse_pattern()?;

        if !self.match_token(TokenKind::Comma) {
            self.consume(TokenKind::RightParen, "')' attendu après le pattern")?;
            return Ok(first);
        }

        let mut elements = vec![first];

        if !self.check(TokenKind::RightParen) {
            loop {
                elements.push(self.parse_pattern()?);

                if !self.match_token(TokenKind::Comma) {
                    break;
                }

                if self.check(TokenKind::RightParen) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RightParen, "')' attendu après le pattern tuple")?;

        Ok(Pattern::Tuple(elements))
    }

    fn parse_array_pattern(&mut self) -> Result<Pattern, ParserError> {
        self.advance(); // '['

        let mut patterns = Vec::new();
        let mut has_rest = false;

        if !self.check(TokenKind::RightBracket) {
            loop {
                if self.match_token(TokenKind::Range) {
                    if has_rest {
                        let token = self.previous().clone();
                        return Err(ParserError {
                            message: "Un seul '..' est autorisé dans un pattern tableau"
                                .to_string(),
                            line: token.line,
                            column: token.column,
                        });
                    }

                    has_rest = true;

                    // Pour Kastel, le rest-pattern est volontairement non
                    // capturant : `[head, ..]`. Un binding de slice pourra
                    // être ajouté plus tard sous une syntaxe dédiée.
                    if !self.check(TokenKind::RightBracket) && !self.check(TokenKind::Comma) {
                        let token = self.peek().clone();
                        return Err(ParserError {
                            message: "Le pattern '..' doit être le dernier élément de '[...]'"
                                .to_string(),
                            line: token.line,
                            column: token.column,
                        });
                    }
                } else {
                    if has_rest {
                        let token = self.peek().clone();
                        return Err(ParserError {
                            message: "Aucun pattern ne peut suivre '..' dans '[...]'".to_string(),
                            line: token.line,
                            column: token.column,
                        });
                    }

                    patterns.push(self.parse_pattern()?);
                }

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

        if has_rest {
            Ok(Pattern::ArrayRest(patterns))
        } else {
            Ok(Pattern::Array(patterns))
        }
    }
}

use crate::error::parse_error::ParserError;
use crate::frontend::ast::*;
use crate::frontend::lexer::token::TokenKind;

use super::Parser;

#[allow(dead_code)]
impl Parser {
    // ============================================================
    // TYPES
    // ============================================================

    pub(super) fn parse_optional_type_annotation(
        &mut self,
    ) -> Result<Option<TypeExpr>, ParserError> {
        if self.match_token(TokenKind::Colon) {
            Ok(Some(self.parse_type_expression()?))
        } else {
            Ok(None)
        }
    }

    pub(super) fn parse_type_expression(&mut self) -> Result<TypeExpr, ParserError> {
        let token = if self.check(TokenKind::Identifier) || self.check(TokenKind::None) {
            self.advance().clone()
        } else {
            return Err(ParserError {
                message: "Nom de type attendu".to_string(),
                line: self.peek().line,
                column: self.peek().column,
            });
        };

        let name = token.lexeme;

        if !self.match_token(TokenKind::Less) {
            return Ok(TypeExpr::Named(name));
        }

        let mut arguments = Vec::new();

        if self.check(TokenKind::Greater) {
            return Err(ParserError {
                message: "Au moins un paramètre de type attendu entre '<' et '>'".to_string(),
                line: self.peek().line,
                column: self.peek().column,
            });
        }

        loop {
            arguments.push(self.parse_type_expression()?);

            if !self.match_token(TokenKind::Comma) {
                break;
            }

            if self.check(TokenKind::Greater) {
                return Err(ParserError {
                    message: "Type attendu après ','".to_string(),
                    line: self.peek().line,
                    column: self.peek().column,
                });
            }
        }

        self.consume_type_greater()?;

        Ok(TypeExpr::Generic { name, arguments })
    }

    /// Le lexer réserve `>>` à l'opérateur de décalage. Dans un type
    /// imbriqué (`Dict<str, Array<int>>`), on le traite donc comme deux
    /// fermetures `>` sans modifier la grammaire des expressions.
    fn consume_type_greater(&mut self) -> Result<(), ParserError> {
        if self.match_token(TokenKind::Greater) {
            return Ok(());
        }

        if self.check(TokenKind::RightShift) {
            let token = self.advance().clone();
            let second = crate::frontend::lexer::token::Token::new(
                TokenKind::Greater,
                ">".to_string(),
                token.line,
                token.column + 1,
            );
            self.tokens.insert(self.current, second);
            return Ok(());
        }

        Err(ParserError {
            message: "'>' attendu après les paramètres de type".to_string(),
            line: self.peek().line,
            column: self.peek().column,
        })
    }

    // ============================================================
    // DECLARATION
    // ============================================================

    pub(super) fn parse_variable_declaration(
        &mut self,
        mutable: bool,
    ) -> Result<Vec<Statement>, ParserError> {
        let mut declarations = Vec::new();

        loop {
            let name = self.consume(TokenKind::Identifier, "Nom de variable attendu")?;

            let type_annotation = self.parse_optional_type_annotation()?;

            self.consume(TokenKind::Equal, "'=' attendu après le nom")?;

            let value = self.parse_expression()?;

            declarations.push(Statement::Let {
                name: name.lexeme,
                value,
                mutable,
                type_annotation,
            });

            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }

        Ok(declarations)
    }
}

use crate::error::parse_error::ParserError;
use crate::frontend::ast::*;
use crate::frontend::lexer::token::TokenKind;

use super::Parser;

#[allow(dead_code)]
impl Parser {
    // ============================================================
    // FUNCTION
    // ============================================================

    pub(super) fn parse_function_statement(&mut self) -> Result<Statement, ParserError> {
        let name = self.consume(TokenKind::Identifier, "Nom de fonction attendu")?;

        self.consume(TokenKind::LeftParen, "'(' attendu après le nom de fonction")?;

        let mut params = Vec::new();
        let mut param_types = Vec::new();

        if !self.check(TokenKind::RightParen) {
            loop {
                let param = self.consume(TokenKind::Identifier, "Nom de paramètre attendu")?;

                params.push(param.lexeme);
                param_types.push(self.parse_optional_type_annotation()?);

                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RightParen, "')' attendu après les paramètres")?;

        let return_type = if self.match_token(TokenKind::Arrow) {
            Some(self.parse_type_expression()?)
        } else {
            None
        };

        self.consume(TokenKind::LeftBrace, "'{' attendu avant le corps")?;

        let body = self.parse_block_statement()?;

        Ok(Statement::Function {
            name: name.lexeme,
            params,
            param_types,
            return_type,
            body,
        })
    }
}
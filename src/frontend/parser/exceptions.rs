use crate::error::parse_error::ParserError;
use crate::frontend::ast::*;
use crate::frontend::lexer::token::TokenKind;

use super::Parser;

impl Parser {
    // ============================================================
    // THROW
    // ============================================================

    pub(super) fn parse_throw_statement(&mut self) -> Result<Statement, ParserError> {
        if self.check(TokenKind::Semicolon) || self.check(TokenKind::RightBrace) || self.is_at_end()
        {
            return Err(ParserError {
                message: "Expression attendue après 'throw'".to_string(),
                line: self.peek().line,
                column: self.peek().column,
            });
        }

        let value = self.parse_expression()?;

        Ok(Statement::Throw { value })
    }

    // ============================================================
    // TRY / CATCH / FINALLY
    // ============================================================

    pub(super) fn parse_try_statement(&mut self) -> Result<Statement, ParserError> {
        /*
         * try {
         *     ...
         * }
         */
        self.consume(TokenKind::LeftBrace, "'{' attendu après 'try'")?;

        let try_body = self.parse_block_statement()?;

        /*
         * catch optionnel.
         *
         * catch (e) {
         *     ...
         * }
         */
        let mut catch_name = None;
        let mut catch_body = None;

        if self.match_token(TokenKind::Catch) {
            self.consume(TokenKind::LeftParen, "'(' attendu après 'catch'")?;

            let error_name = self.consume(
                TokenKind::Identifier,
                "Nom de variable attendu dans 'catch'",
            )?;

            self.consume(
                TokenKind::RightParen,
                "')' attendu après le nom de l'erreur",
            )?;

            self.consume(TokenKind::LeftBrace, "'{' attendu après 'catch(...)'")?;

            let body = self.parse_block_statement()?;

            catch_name = Some(error_name.lexeme);
            catch_body = Some(body);
        }

        /*
         * finally optionnel.
         *
         * finally {
         *     ...
         * }
         */
        let finally_body = if self.match_token(TokenKind::Finally) {
            self.consume(TokenKind::LeftBrace, "'{' attendu après 'finally'")?;

            Some(self.parse_block_statement()?)
        } else {
            None
        };

        /*
         * Il faut au moins catch ou finally.
         *
         * Ceci est invalide :
         *
         * try {
         *     ...
         * }
         */
        if catch_body.is_none() && finally_body.is_none() {
            return Err(ParserError {
                message: "'try' doit être suivi de 'catch' ou 'finally'".to_string(),
                line: self.previous().line,
                column: self.previous().column,
            });
        }

        Ok(Statement::Try {
            try_body,
            catch_name,
            catch_body,
            finally_body,
        })
    }
}

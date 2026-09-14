use crate::error::parse_error::ParserError;
use crate::frontend::ast::*;
use crate::frontend::token::*;

use super::Parser;

#[allow(dead_code)]
impl Parser {
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
}
use crate::error::parse_error::ParserError;
use crate::frontend::ast::*;
use crate::frontend::lexer::token::TokenKind;


use super::Parser;

#[allow(dead_code)]
impl Parser {
    // ============================================================
    // CLASS
    // ============================================================

    pub(super) fn parse_class_statement(&mut self) -> Result<Statement, ParserError> {
        let name = self.consume(TokenKind::Identifier, "Nom de classe attendu après 'class'")?;
        let mut bases = Vec::new();

        if self.match_token(TokenKind::Colon) {
            loop {
                let base = self.consume(
                    TokenKind::Identifier,
                    "Nom de classe ou d'interface attendu après ':'",
                )?;

                bases.push(base.lexeme);

                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenKind::LeftBrace, "'{' attendu après le nom de classe")?;

        let mut methods = Vec::new();

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            self.consume(
                TokenKind::Function,
                "'function' attendu dans le corps de la classe",
            )?;

            let method_name = self.consume(TokenKind::Identifier, "Nom de méthode attendu")?;

            self.consume(TokenKind::LeftParen, "'(' attendu après le nom de méthode")?;

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

                    if self.check(TokenKind::RightParen) {
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

            self.consume(
                TokenKind::LeftBrace,
                "'{' attendu avant le corps de la méthode",
            )?;

            let body = self.parse_block_statement()?;

            methods.push(FunctionMethod {
                name: method_name.lexeme,
                params,
                param_types,
                return_type,
                body,
            });
        }

        self.consume(
            TokenKind::RightBrace,
            "'}' attendu après le corps de la classe",
        )?;

        Ok(Statement::Class {
            name: name.lexeme,
            bases,
            methods,
        })
    }

    pub(super) fn parse_interface_statement(&mut self) -> Result<Statement, ParserError> {
        let name = self.consume(
            TokenKind::Identifier,
            "Nom d'interface attendu après 'interface'",
        )?;

        // ============================================================
        // INTERFACES PARENTES
        // ============================================================

        let mut bases = Vec::new();

        if self.match_token(TokenKind::Colon) {
            loop {
                let base = self.consume(
                    TokenKind::Identifier,
                    "Nom d'interface parent attendu après ':'",
                )?;

                bases.push(base.lexeme);

                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        // ============================================================
        // CORPS
        // ============================================================

        self.consume(
            TokenKind::LeftBrace,
            "'{' attendu après le nom de l'interface",
        )?;

        let mut methods = Vec::new();

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            self.consume(TokenKind::Function, "'function' attendu dans l'interface")?;

            let method_name = self.consume(
                TokenKind::Identifier,
                "Nom de méthode attendu dans l'interface",
            )?;

            self.consume(TokenKind::LeftParen, "'(' attendu après le nom de méthode")?;

            let mut arity = 0;

            if !self.check(TokenKind::RightParen) {
                loop {
                    self.consume(
                        TokenKind::Identifier,
                        "Nom de paramètre attendu dans l'interface",
                    )?;

                    arity += 1;

                    if !self.match_token(TokenKind::Comma) {
                        break;
                    }

                    if self.check(TokenKind::RightParen) {
                        break;
                    }
                }
            }

            self.consume(TokenKind::RightParen, "')' attendu après les paramètres")?;

            self.consume(
                TokenKind::Semicolon,
                "';' attendu après la signature de méthode",
            )?;

            methods.push(InterfaceMethod {
                name: method_name.lexeme,
                arity,
            });
        }

        self.consume(
            TokenKind::RightBrace,
            "'}' attendu après le corps de l'interface",
        )?;

        Ok(Statement::Interface {
            name: name.lexeme,
            bases,
            methods,
        })
    }
}
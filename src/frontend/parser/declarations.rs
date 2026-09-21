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

    /// Type complet : un type simple, paramétré ou objet, éventuellement en
    /// UNION (`int | float`).
    pub(super) fn parse_type_expression(&mut self) -> Result<TypeExpr, ParserError> {
        let first = self.parse_type_primary()?;

        if !self.check(TokenKind::Pipe) {
            return Ok(first);
        }

        let mut members = vec![first];

        while self.match_token(TokenKind::Pipe) {
            members.push(self.parse_type_primary()?);
        }

        Ok(TypeExpr::Union(members))
    }

    /// `{ name: str, age: int }`
    fn parse_record_type(&mut self) -> Result<TypeExpr, ParserError> {
        self.consume(TokenKind::LeftBrace, "'{' attendu")?;

        let mut fields: Vec<(String, TypeExpr)> = Vec::new();

        if !self.check(TokenKind::RightBrace) {
            loop {
                let name = self.consume(TokenKind::Identifier, "Nom de champ attendu")?;

                self.consume(TokenKind::Colon, "':' attendu après le nom du champ")?;

                let field_type = self.parse_type_expression()?;

                if fields.iter().any(|(existing, _)| *existing == name.lexeme) {
                    return Err(ParserError {
                        message: format!("Le champ '{}' est déjà déclaré dans ce type", name.lexeme),
                        line: name.line,
                        column: name.column,
                    });
                }

                fields.push((name.lexeme, field_type));

                if !self.match_token(TokenKind::Comma) {
                    break;
                }

                if self.check(TokenKind::RightBrace) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RightBrace, "'}' attendu après le type objet")?;

        Ok(TypeExpr::Record(fields))
    }

    fn parse_type_primary(&mut self) -> Result<TypeExpr, ParserError> {
        if self.check(TokenKind::LeftBrace) {
            return self.parse_record_type();
        }

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

        // `Array` a été renommé `List`.
        if name.eq_ignore_ascii_case("array") {
            return Err(ParserError {
                message: "Le type 'Array' a été renommé 'List' : écrivez List<T>".to_string(),
                line: token.line,
                column: token.column,
            });
        }

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

        // `let v: Array<int>= [1, 2];` : le lexer produit `>=` ; on le
        // scinde en `>` (fermeture du type) suivi de `=` (initialisation).
        if self.check(TokenKind::GreaterEqual) {
            let token = self.advance().clone();
            let equal = crate::frontend::lexer::token::Token::new(
                TokenKind::Equal,
                "=".to_string(),
                token.line,
                token.column + 1,
            );
            self.tokens.insert(self.current, equal);
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

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
                        message: format!(
                            "Le champ '{}' est déjà déclaré dans ce type",
                            name.lexeme
                        ),
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

    /// Parse un type parenthésé ou un type tuple :
    /// `()`, `(int,)`, `(int, str)`, `(Result<int, str>, List<float>)`.
    /// Sans virgule, `(int)` reste un simple groupement du type `int`.
    fn parse_parenthesized_or_tuple_type(&mut self) -> Result<TypeExpr, ParserError> {
        self.consume(TokenKind::LeftParen, "'(' attendu")?;

        if self.match_token(TokenKind::RightParen) {
            return Ok(TypeExpr::Tuple(Vec::new()));
        }

        let first = self.parse_type_expression()?;

        if !self.match_token(TokenKind::Comma) {
            self.consume(TokenKind::RightParen, "')' attendu après le type")?;
            return Ok(first);
        }

        let mut elements = vec![first];

        if !self.check(TokenKind::RightParen) {
            loop {
                elements.push(self.parse_type_expression()?);

                if !self.match_token(TokenKind::Comma) {
                    break;
                }

                if self.check(TokenKind::RightParen) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RightParen, "')' attendu après le type tuple")?;
        Ok(TypeExpr::Tuple(elements))
    }

    fn parse_type_primary(&mut self) -> Result<TypeExpr, ParserError> {
        if self.check(TokenKind::LeftParen) {
            return self.parse_parenthesized_or_tuple_type();
        }

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
    pub(super) fn consume_type_greater(&mut self) -> Result<(), ParserError> {
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

    /// Parse les paramètres génériques d'une déclaration : `<T, U>`.
    /// Une ou plusieurs contraintes sont possibles : `<T: Add + Eq>` ou `<T: Comparable>`.
    pub(super) fn parse_generic_parameters(&mut self) -> Result<Vec<GenericParam>, ParserError> {
        if !self.match_token(TokenKind::Less) {
            return Ok(Vec::new());
        }

        let mut parameters = Vec::new();

        loop {
            let parameter =
                self.consume(TokenKind::Identifier, "Nom de paramètre générique attendu")?;

            if parameters
                .iter()
                .any(|existing: &GenericParam| existing.name == parameter.lexeme)
            {
                return Err(ParserError {
                    message: format!(
                        "Le paramètre générique '{}' est déjà déclaré",
                        parameter.lexeme
                    ),
                    line: parameter.line,
                    column: parameter.column,
                });
            }

            let mut bounds = Vec::new();
            if self.match_token(TokenKind::Colon) {
                loop {
                    bounds.push(self.parse_type_expression()?);

                    if !self.match_token(TokenKind::Plus) {
                        break;
                    }
                }
            }

            parameters.push(GenericParam {
                name: parameter.lexeme,
                bounds,
            });

            if !self.match_token(TokenKind::Comma) {
                break;
            }

            if self.check(TokenKind::Greater) || self.check(TokenKind::RightShift) {
                return Err(ParserError {
                    message: "Paramètre générique attendu après ','".to_string(),
                    line: self.peek().line,
                    column: self.peek().column,
                });
            }
        }

        self.consume_type_greater()?;
        Ok(parameters)
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

// src/frontend/parser/declarations.rs
// Remplace entièrement le module #[cfg(test)] actuel par celui-ci.

#[cfg(test)]
mod tuple_type_tests {
    use crate::frontend::{
        ast::{Statement, TypeExpr},
        lexer::lexer::Lexer,
    };

    use super::Parser;

    fn unwrap_function(statement: &Statement) -> &Statement {
        match statement {
            Statement::Positioned { statement, .. } => unwrap_function(statement),
            statement => statement,
        }
    }

    #[test]
    fn parses_tuple_types_in_parameters_and_return_types() {
        let source = r#"
            func pair(value: (int, str)) -> (str, int) {
                return ("ok", 42);
            }
        "#;

        let tokens = Lexer::new(source.to_string())
            .scan_token()
            .expect("lexer should accept tuple type syntax");

        let mut parser = Parser::new(tokens);
        let statements = parser
            .parse()
            .expect("parser should accept tuple types");

        let Statement::Function {
            param_types,
            return_type,
            ..
        } = unwrap_function(&statements[0])
        else {
            panic!("expected function declaration");
        };

        assert_eq!(
            param_types[0],
            Some(TypeExpr::Tuple(vec![
                TypeExpr::Named("int".to_string()),
                TypeExpr::Named("str".to_string()),
            ]))
        );

        assert_eq!(
            return_type,
            &Some(TypeExpr::Tuple(vec![
                TypeExpr::Named("str".to_string()),
                TypeExpr::Named("int".to_string()),
            ]))
        );
    }

    #[test]
    fn parses_empty_and_singleton_tuple_types() {
        let source = r#"
            func empty(value: ()) -> () {
                return ();
            }

            func single(value: (int,)) -> (str,) {
                return ("ok",);
            }
        "#;

        let tokens = Lexer::new(source.to_string())
            .scan_token()
            .expect("lexer should accept tuple type syntax");

        let mut parser = Parser::new(tokens);
        let statements = parser
            .parse()
            .expect("parser should accept tuple types");

        let Statement::Function {
            param_types,
            return_type,
            ..
        } = unwrap_function(&statements[0])
        else {
            panic!("expected first function");
        };

        assert_eq!(
            param_types[0],
            Some(TypeExpr::Tuple(Vec::new()))
        );

        assert_eq!(
            return_type,
            &Some(TypeExpr::Tuple(Vec::new()))
        );

        let Statement::Function {
            param_types,
            return_type,
            ..
        } = unwrap_function(&statements[1])
        else {
            panic!("expected second function");
        };

        assert_eq!(
            param_types[0],
            Some(TypeExpr::Tuple(vec![
                TypeExpr::Named("int".to_string())
            ]))
        );

        assert_eq!(
            return_type,
            &Some(TypeExpr::Tuple(vec![
                TypeExpr::Named("str".to_string())
            ]))
        );
    }

    #[test]
    fn parses_tuple_types_inside_generic_types() {
        let source = r#"
            func values() -> List<(int, str)> {
                return [];
            }

            func result() -> Result<(float, float), str> {
                return Ok((1.0, 2.0));
            }
        "#;

        let tokens = Lexer::new(source.to_string())
            .scan_token()
            .expect("lexer should accept generic tuple types");

        let mut parser = Parser::new(tokens);
        let statements = parser
            .parse()
            .expect("parser should accept generic tuple types");

        let Statement::Function {
            return_type,
            ..
        } = unwrap_function(&statements[0])
        else {
            panic!("expected first function");
        };

        assert_eq!(
            return_type,
            &Some(TypeExpr::Generic {
                name: "List".to_string(),
                arguments: vec![TypeExpr::Tuple(vec![
                    TypeExpr::Named("int".to_string()),
                    TypeExpr::Named("str".to_string()),
                ])],
            })
        );

        let Statement::Function {
            return_type,
            ..
        } = unwrap_function(&statements[1])
        else {
            panic!("expected second function");
        };

        assert_eq!(
            return_type,
            &Some(TypeExpr::Generic {
                name: "Result".to_string(),
                arguments: vec![
                    TypeExpr::Tuple(vec![
                        TypeExpr::Named("float".to_string()),
                        TypeExpr::Named("float".to_string()),
                    ]),
                    TypeExpr::Named("str".to_string()),
                ],
            })
        );
    }
}
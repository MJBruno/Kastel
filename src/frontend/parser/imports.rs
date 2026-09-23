use crate::error::parse_error::ParserError;
use crate::frontend::ast::*;
use crate::frontend::lexer::token::TokenKind;

use super::Parser;

#[allow(dead_code)]
impl Parser {
    // ============================================================
    // IMPORT
    // ============================================================

    pub(super) fn parse_import_statement(&mut self) -> Result<Statement, ParserError> {
        // ------------------------------------------------------------
        // import * from export_mod;
        // ------------------------------------------------------------
        if self.match_token(TokenKind::Star) {
            self.consume(TokenKind::From, "'from' attendu après '*'")?;

            let module = self.parse_module_path()?;

            return Ok(Statement::FromImport {
                module,
                items: vec![ImportItem {
                    name: "*".to_string(),
                    alias: None,
                }],
            });
        }

        // ------------------------------------------------------------
        // import { double, answer } from export_mod;
        // ------------------------------------------------------------
        if self.check(TokenKind::LeftBrace) {
            let items = self.parse_import_item_list()?;

            self.consume(TokenKind::From, "'from' attendu après la liste d'import")?;

            let module = self.parse_module_path()?;

            return Ok(Statement::FromImport { module, items });
        }

        let module = self.parse_module_path()?;

        // ------------------------------------------------------------
        // import dog;
        // ------------------------------------------------------------
        if !self.check(TokenKind::LeftBrace) {
            return Ok(Statement::Import { path: module.parts });
        }

        // ------------------------------------------------------------
        // import dog { Dog, Animal, details };
        // ------------------------------------------------------------

        let items = self.parse_import_item_list()?;

        Ok(Statement::FromImport { module, items })
    }

    /// Parse `{ nom (as alias)?, ... }`, partagé par les trois syntaxes
    /// d'import qui acceptent une liste entre accolades :
    ///   - `import module { a, b as c };`
    ///   - `from module import { a, b as c };`
    ///   - `import { a, b as c } from module;`
    fn parse_import_item_list(&mut self) -> Result<Vec<ImportItem>, ParserError> {
        self.consume(TokenKind::LeftBrace, "'{' attendu")?;

        let mut items = Vec::new();

        if self.check(TokenKind::RightBrace) {
            return Err(ParserError {
                message: "La liste d'import ne peut pas être vide".to_string(),
                line: self.peek().line,
                column: self.peek().column,
            });
        }

        loop {
            let name = self.consume(TokenKind::Identifier, "Nom exporté attendu")?;

            let alias = if self.match_token(TokenKind::As) {
                Some(
                    self.consume(TokenKind::Identifier, "Alias attendu après 'as'")?
                        .lexeme,
                )
            } else {
                None
            };

            items.push(ImportItem {
                name: name.lexeme,
                alias,
            });

            if !self.match_token(TokenKind::Comma) {
                break;
            }

            if self.check(TokenKind::RightBrace) {
                return Err(ParserError {
                    message: "Nom exporté attendu après ','".to_string(),
                    line: self.peek().line,
                    column: self.peek().column,
                });
            }
        }

        self.consume(
            TokenKind::RightBrace,
            "'}' attendu après la liste des imports",
        )?;

        Ok(items)
    }

    pub(super) fn parse_from_import_statement(&mut self) -> Result<Statement, ParserError> {
        let module = self.parse_module_path()?;

        self.consume(TokenKind::Import, "'import' attendu après le nom du module")?;

        // ------------------------------------------------------------
        // from dog import *
        // ------------------------------------------------------------

        if self.match_token(TokenKind::Star) {
            return Ok(Statement::FromImport {
                module,
                items: vec![ImportItem {
                    name: "*".to_string(),
                    alias: None,
                }],
            });
        }

        // ------------------------------------------------------------
        // from dog import { Dog, Animal, details };
        // ------------------------------------------------------------

        if self.check(TokenKind::LeftBrace) {
            let items = self.parse_import_item_list()?;

            return Ok(Statement::FromImport { module, items });
        }

        // ------------------------------------------------------------
        // from dog import Dog, Animal, details
        // ------------------------------------------------------------

        let mut items = Vec::new();

        loop {
            let name = self.consume(TokenKind::Identifier, "Nom exporté attendu")?;

            let alias = if self.match_token(TokenKind::As) {
                Some(
                    self.consume(TokenKind::Identifier, "Alias attendu après 'as'")?
                        .lexeme,
                )
            } else {
                None
            };

            items.push(ImportItem {
                name: name.lexeme,
                alias,
            });

            if !self.match_token(TokenKind::Comma) {
                break;
            }

            if self.check(TokenKind::Star) {
                return Err(ParserError {
                    message: "'*' ne peut pas être combiné avec d'autres imports".to_string(),
                    line: self.peek().line,
                    column: self.peek().column,
                });
            }
        }

        Ok(Statement::FromImport { module, items })
    }

    pub(super) fn parse_export_statement(&mut self) -> Result<Statement, ParserError> {
        let statement = if self.match_token(TokenKind::Let) {
            let mut declarations = self.parse_variable_declaration(true)?;

            if declarations.len() != 1 {
                return Err(ParserError {
                    message: "'export' accepte une seule déclaration".to_string(),
                    line: self.previous().line,
                    column: self.previous().column,
                });
            }

            declarations.remove(0)
        } else if self.match_token(TokenKind::Const) {
            let mut declarations = self.parse_variable_declaration(false)?;

            if declarations.len() != 1 {
                return Err(ParserError {
                    message: "'export' accepte une seule déclaration".to_string(),
                    line: self.previous().line,
                    column: self.previous().column,
                });
            }

            declarations.remove(0)
        } else if self.match_token(TokenKind::Function) {
            self.parse_function_statement()?
        } else if self.match_token(TokenKind::Class) {
            self.parse_class_statement()?
        } else if self.match_token(TokenKind::Interface) {
            self.parse_interface_statement()?
        } else if self.check(TokenKind::Identifier) && self.peek().lexeme == "type" {
            // `export type Person = { ... };` : rend l'alias visible aux
            // AUTRES modules (voir `ModuleTypeInterface::type_aliases`). Un
            // alias non exporté reste local au fichier qui le déclare.
            self.parse_type_alias_statement()?
        } else {
            return Err(ParserError {
                message:
                    "'export' doit être suivi de let, const, function, class, interface ou type"
                        .to_string(),
                line: self.peek().line,
                column: self.peek().column,
            });
        };

        Ok(Statement::Export {
            statement: Box::new(statement),
        })
    }

    fn parse_module_path(&mut self) -> Result<ModulePath, ParserError> {
        let first = self.consume(TokenKind::Identifier, "Nom de module attendu")?;

        let mut parts = vec![first.lexeme];

        while self.match_token(TokenKind::Dot) {
            let part = self.consume(TokenKind::Identifier, "Nom de module attendu après '.'")?;

            parts.push(part.lexeme);
        }

        Ok(ModulePath::new(parts))
    }
}

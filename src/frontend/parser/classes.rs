use crate::error::parse_error::ParserError;
use crate::frontend::ast::*;
use crate::frontend::lexer::token::TokenKind;


use super::Parser;

/// Nom du constructeur reconnu par la VM.
const CONSTRUCTOR_NAME: &str = "init";

/// Alias accepté dans le code source : `func initialize(...)`.
const CONSTRUCTOR_ALIAS: &str = "initialize";

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

        let mut fields: Vec<ClassField> = Vec::new();
        let mut methods = Vec::new();

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            // Modificateur optionnel : `public` / `private`.
            let visibility = self.parse_member_visibility();

            // `private let age: int = 0;`
            if self.check(TokenKind::Let) {
                let field = self.parse_class_field(visibility)?;

                if fields.iter().any(|existing| existing.name == field.name) {
                    return Err(ParserError {
                        message: format!(
                            "Le champ '{}' est déjà déclaré dans la classe '{}'",
                            field.name, name.lexeme
                        ),
                        line: field.line,
                        column: field.column,
                    });
                }

                fields.push(field);
                continue;
            }

            self.consume(
                TokenKind::Function,
                "'func' ou 'let' attendu dans le corps de la classe",
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

            // `initialize` est un alias du constructeur `init` : la VM et le
            // vérificateur de types ne connaissent que `init`.
            let normalized_name = if method_name.lexeme == CONSTRUCTOR_ALIAS {
                CONSTRUCTOR_NAME.to_string()
            } else {
                method_name.lexeme
            };

            methods.push(FunctionMethod {
                name: normalized_name,
                visibility,
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

        // La visibilité est portée par le NOM de la méthode (comme la VM la
        // stocke) : toutes les surcharges d'un même nom doivent donc être
        // aussi publiques, ou toutes privées.
        for (index, method) in methods.iter().enumerate() {
            if methods[..index]
                .iter()
                .any(|other| other.name == method.name && other.visibility != method.visibility)
            {
                return Err(ParserError {
                    message: format!(
                        "Les surcharges de la méthode '{}' doivent avoir la même visibilité",
                        method.name
                    ),
                    line: name.line,
                    column: name.column,
                });
            }
        }

        let class_name = name.lexeme;

        Self::desugar_field_initializers(&class_name, &fields, &mut methods);

        Ok(Statement::Class {
            name: class_name,
            bases,
            fields,
            methods,
        })
    }

    // ============================================================
    // MEMBRES DE CLASSE : VISIBILITÉ ET CHAMPS
    // ============================================================

    /// Lit un modificateur `public` / `private` s'il est suivi de `let` ou de
    /// `func`. Ce sont des mots-clés CONTEXTUELS : hors du corps d'une classe
    /// (ou sans `let`/`func` derrière), `public` et `private` restent de
    /// simples identifiants et aucun programme existant ne casse.
    fn parse_member_visibility(&mut self) -> Visibility {
        let is_modifier = self.check(TokenKind::Identifier)
            && matches!(self.peek().lexeme.as_str(), "public" | "private")
            && (self.check_next(TokenKind::Let) || self.check_next(TokenKind::Function));

        if !is_modifier {
            return Visibility::Public;
        }

        let visibility = if self.peek().lexeme == "private" {
            Visibility::Private
        } else {
            Visibility::Public
        };

        self.advance();

        visibility
    }

    /// `let nom: type = valeur;` (annotation et valeur optionnelles).
    fn parse_class_field(&mut self, visibility: Visibility) -> Result<ClassField, ParserError> {
        self.consume(TokenKind::Let, "'let' attendu")?;

        let name = self.consume(TokenKind::Identifier, "Nom de champ attendu après 'let'")?;
        let type_annotation = self.parse_optional_type_annotation()?;

        let initializer = if self.match_token(TokenKind::Equal) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.match_token(TokenKind::Semicolon);

        Ok(ClassField {
            name: name.lexeme,
            visibility,
            type_annotation,
            initializer,
            line: name.line,
            column: name.column,
        })
    }

    /// Transforme les valeurs initiales des champs en code ordinaire :
    ///
    /// ```text
    /// private let age: int = 0;
    ///
    /// func __fields_Personne() { this.age = 0; }      // méthode cachée
    /// func init(...) { this.__fields_Personne(); ... }  // ajout en tête
    /// ```
    ///
    /// La VM n'a donc rien à savoir des initialiseurs. Le nom de la méthode
    /// cachée contient celui de la classe pour qu'une classe dérivée ne
    /// masque pas les initialiseurs de sa classe de base. Sans `init`
    /// explicite, un `init()` sans paramètre est ajouté.
    fn desugar_field_initializers(
        class_name: &str,
        fields: &[ClassField],
        methods: &mut Vec<FunctionMethod>,
    ) {
        let initializers: Vec<Statement> = fields
            .iter()
            .filter_map(|field| {
                let value = field.initializer.clone()?;

                Some(Statement::Positioned {
                    line: field.line,
                    column: field.column,
                    statement: Box::new(Statement::Assignment {
                        target: AssignmentTarget::Member {
                            object: Box::new(Expression::This),
                            name: field.name.clone(),
                        },
                        value,
                    }),
                })
            })
            .collect();

        if initializers.is_empty() {
            return;
        }

        let helper = format!("__fields_{class_name}");

        let call_helper = |helper: &str| Statement::Expression {
            expression: Expression::Call {
                callee: Box::new(Expression::Member {
                    object: Box::new(Expression::This),
                    name: helper.to_string(),
                    line: 0,
                    column: 0,
                }),
                arguments: Vec::new(),
                line: 0,
                column: 0,
            },
        };

        let mut has_constructor = false;

        for method in methods.iter_mut() {
            if method.name == CONSTRUCTOR_NAME {
                has_constructor = true;
                method.body.insert(0, call_helper(&helper));
            }
        }

        if !has_constructor {
            methods.push(FunctionMethod {
                name: CONSTRUCTOR_NAME.to_string(),
                visibility: Visibility::Public,
                params: Vec::new(),
                param_types: Vec::new(),
                return_type: None,
                body: vec![call_helper(&helper)],
            });
        }

        methods.push(FunctionMethod {
            name: helper,
            visibility: Visibility::Private,
            params: Vec::new(),
            param_types: Vec::new(),
            return_type: None,
            body: initializers,
        });
    }

    pub(super) fn parse_interface_statement(&mut self) -> Result<Statement, ParserError> {
        let name = self.consume(
            TokenKind::Identifier,
            "Nom d'interface attendu après 'interface'",
        )?;

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

            let mut params = Vec::new();
            let mut param_types = Vec::new();

            if !self.check(TokenKind::RightParen) {
                loop {
                    let parameter = self.consume(
                        TokenKind::Identifier,
                        "Nom de paramètre attendu dans l'interface",
                    )?;

                    params.push(parameter.lexeme);
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
                TokenKind::Semicolon,
                "';' attendu après la signature de méthode",
            )?;

            methods.push(InterfaceMethod {
                name: method_name.lexeme,
                arity: params.len(),
                params,
                param_types,
                return_type,
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
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

        let mut fields: Vec<ClassField> = Vec::new();
        let mut methods = Vec::new();

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            // Modificateurs optionnels : `public`/`protected`/`private` (visibilité) et
            // `static` (portée), dans n'importe quel ordre.
            let (visibility, is_static) = self.parse_member_modifiers();

            // `private let age: int = 0;` / `static let compteur: int = 0;`
            if self.check(TokenKind::Let) {
                let field = self.parse_class_field(visibility, is_static)?;

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

            // L'ancien constructeur `init` n'est plus reconnu : plutôt que de
            // le traiter silencieusement comme une méthode ordinaire (et de
            // faire échouer `new` avec une erreur d'arité déroutante), on
            // indique la migration à faire.
            if method_name.lexeme == LEGACY_CONSTRUCTOR_NAME {
                return Err(ParserError {
                    message: format!(
                        "Le constructeur s'appelle désormais '{CONSTRUCTOR_NAME}' : \
                         renommez 'func {LEGACY_CONSTRUCTOR_NAME}(...)' en \
                         'func {CONSTRUCTOR_NAME}(...)'"
                    ),
                    line: method_name.line,
                    column: method_name.column,
                });
            }

            // Un constructeur n'a pas de sens sans instance à construire.
            if is_static && method_name.lexeme == CONSTRUCTOR_NAME {
                return Err(ParserError {
                    message: format!(
                        "Le constructeur '{CONSTRUCTOR_NAME}' ne peut pas être 'static'"
                    ),
                    line: method_name.line,
                    column: method_name.column,
                });
            }

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
                visibility,
                is_static,
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

        // La visibilité (et le caractère `static`) sont portés par le NOM de
        // la méthode (comme la VM les stocke) : toutes les surcharges d'un
        // même nom doivent donc partager la même visibilité et la même
        // portée.
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

            if methods[..index]
                .iter()
                .any(|other| other.name == method.name && other.is_static != method.is_static)
            {
                return Err(ParserError {
                    message: format!(
                        "Les surcharges de la méthode '{}' doivent être toutes 'static', ou \
                         toutes non-'static'",
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

    /// Lit les modificateurs `public`/`protected`/`private` (visibilité) et `static`
    /// (portée) d'un membre, dans n'importe quel ordre, chacun facultatif.
    /// Ce sont des mots-clés CONTEXTUELS, comme `public`/`private` déjà :
    /// hors du corps d'une classe (ou sans `let`/`func` au bout de la
    /// séquence de modificateurs), ils restent de simples identifiants et
    /// aucun programme existant ne casse.
    fn parse_member_modifiers(&mut self) -> (Visibility, bool) {
        let mut visibility = Visibility::Public;
        let mut is_static = false;

        // Au plus deux modificateurs ont un sens ("public static" /
        // "static private" / ...) : un troisième serait forcément une
        // répétition, donc plus un modificateur valide.
        for _ in 0..2 {
            if !self.check(TokenKind::Identifier) {
                break;
            }

            let lexeme = self.peek().lexeme.clone();

            if !matches!(lexeme.as_str(), "public" | "protected" | "private" | "static") {
                break;
            }

            // Ce mot n'est un modificateur que s'il est directement suivi de
            // 'let'/'func', ou d'un second modificateur lui-même suivi de
            // 'let'/'func'.
            let followed_by_member_start = self.check_next(TokenKind::Let)
                || self.check_next(TokenKind::Function)
                || (self.current + 2 < self.tokens.len()
                    && self.tokens[self.current + 1].kind == TokenKind::Identifier
                    && matches!(
                        self.tokens[self.current + 1].lexeme.as_str(),
                        "public" | "protected" | "private" | "static"
                    )
                    && matches!(
                        self.tokens[self.current + 2].kind,
                        TokenKind::Let | TokenKind::Function
                    ));

            if !followed_by_member_start {
                break;
            }

            match lexeme.as_str() {
                "static" => is_static = true,
                "protected" => visibility = Visibility::Protected,
                "private" => visibility = Visibility::Private,
                "public" => visibility = Visibility::Public,
                _ => unreachable!(),
            }

            self.advance();
        }

        (visibility, is_static)
    }

    /// `let nom: type = valeur;` (annotation et valeur optionnelles).
    fn parse_class_field(
        &mut self,
        visibility: Visibility,
        is_static: bool,
    ) -> Result<ClassField, ParserError> {
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
            is_static,
            type_annotation,
            initializer,
            line: name.line,
            column: name.column,
        })
    }

    /// Transforme les valeurs initiales des champs D'INSTANCE en méthode
    /// cachée :
    ///
    /// ```text
    /// private let age: int = 0;
    ///
    /// private func __fields_Personne() { this.age = 0; }
    /// ```
    ///
    /// La VM l'exécute à chaque `new`, AVANT le constructeur et en commençant
    /// par la classe de base : les champs sont donc initialisés que la classe
    /// déclare un `initialize`, en hérite, ou utilise le constructeur par
    /// défaut implicite. Le nom contient celui de la classe pour qu'une
    /// classe dérivée ne masque pas les initialiseurs de sa base.
    ///
    /// Les champs `static` n'ont PAS de `this` : leur valeur initiale est
    /// évaluée directement par `compile_class`, une seule fois, à la
    /// déclaration de la classe (voir `compiler::statements::compile_class`).
    fn desugar_field_initializers(
        class_name: &str,
        fields: &[ClassField],
        methods: &mut Vec<FunctionMethod>,
    ) {
        let initializers: Vec<Statement> = fields
            .iter()
            .filter(|field| !field.is_static)
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

        methods.push(FunctionMethod {
            name: format!("{FIELD_INITIALIZER_PREFIX}{class_name}"),
            visibility: Visibility::Private,
            is_static: false,
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

    // ============================================================
    // ENUM
    // ============================================================

    /// Parse un enum :
    ///
    /// ```text
    /// enum Color {
    ///     Red,
    ///     Green,
    ///     Blue,
    ///
    ///     func is_blue() -> bool {
    ///         return this == Color.Blue;
    ///     }
    /// }
    /// ```
    ///
    /// Les variants sont toujours qualifiés (`Color.Red`) afin d'éviter les
    /// collisions entre enums. Les méthodes suivent les mêmes règles de
    /// paramètres, surcharge et `this` que les méthodes de classe.
    pub(super) fn parse_enum_statement(&mut self) -> Result<Statement, ParserError> {
        let name = self.consume(TokenKind::Identifier, "Nom d'enum attendu après 'enum'")?;
        self.consume(
            TokenKind::LeftBrace,
            "'{' attendu après le nom de l'enum",
        )?;

        let mut variants = Vec::new();
        let mut methods = Vec::new();
        let mut methods_started = false;

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            if self.match_token(TokenKind::Function) {
                methods_started = true;
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
                    visibility: Visibility::Public,
                    is_static: false,
                    params,
                    param_types,
                    return_type,
                    body,
                });

                // Une virgule après une méthode est facultative, comme celle
                // après un variant.
                self.match_token(TokenKind::Comma);
                continue;
            }

            if methods_started {
                return Err(ParserError {
                    message: format!(
                        "Les variants de l'enum '{}' doivent être déclarés avant ses méthodes",
                        name.lexeme
                    ),
                    line: self.peek().line,
                    column: self.peek().column,
                });
            }

            let variant = self.consume(TokenKind::Identifier, "Variant d'enum attendu")?;

            if variants.iter().any(|existing| existing == &variant.lexeme) {
                return Err(ParserError {
                    message: format!(
                        "Le variant '{}' est déjà déclaré dans l'enum '{}'",
                        variant.lexeme, name.lexeme
                    ),
                    line: variant.line,
                    column: variant.column,
                });
            }

            variants.push(variant.lexeme);

            // Une virgule sépare deux variants. Elle est facultative uniquement
            // avant la fin de l'enum ou avant sa première méthode.
            if self.match_token(TokenKind::Comma) {
                // séparateur consommé
            } else if !self.check(TokenKind::RightBrace) && !self.check(TokenKind::Function) {
                return Err(ParserError {
                    message: "',' attendu entre les variants de l'enum".to_string(),
                    line: self.peek().line,
                    column: self.peek().column,
                });
            }
        }

        if variants.is_empty() {
            return Err(ParserError {
                message: format!("L'enum '{}' doit contenir au moins un variant", name.lexeme),
                line: name.line,
                column: name.column,
            });
        }

        self.consume(
            TokenKind::RightBrace,
            "'}' attendu après le corps de l'enum",
        )?;

        // Comme pour les classes, les surcharges sont distinguées par leur
        // arité ; une même arité doit rester unique.
        for index in 0..methods.len() {
            if methods[..index].iter().any(|other: &FunctionMethod| {
                other.name == methods[index].name
                    && other.params.len() == methods[index].params.len()
            }) {
                return Err(ParserError {
                    message: format!(
                        "La méthode '{}' est déjà déclarée avec {} paramètre(s) dans l'enum '{}'",
                        methods[index].name,
                        methods[index].params.len(),
                        name.lexeme
                    ),
                    line: name.line,
                    column: name.column,
                });
            }
        }

        Ok(Statement::Enum {
            name: name.lexeme,
            variants,
            methods,
        })
    }

}

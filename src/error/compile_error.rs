use crate::error::lex_error::LexerError;
use crate::error::parse_error::ParserError;
use crate::error::runtime_error::RuntimeError;

// ================================================================
// COMPILE_ERROR
// ================================================================
//
// Un seul type, comme avant — AUCUN changement de signature nécessaire
// dans tout le reste du compilateur (compile_expression, compile_var,
// resolve_variable, etc. gardent tous `Result<_, CompileError>`).
//
// La position (ligne, colonne) est ajoutée via UNE SEULE variante
// enveloppante, `WithLocation`, posée UNE SEULE FOIS au tout dernier
// moment — juste avant que l'erreur ne s'échappe vers l'appelant, dans
// les 3 points d'entrée du compilateur (`compile`, `compile_module`,
// `compile_function`). Le reste du compilateur continue de construire et
// de propager des `CompileError` bruts via `?`, exactement comme avant :
// zéro ripple sur les ~40 signatures existantes.

#[derive(Debug)]
#[allow(dead_code)]
pub enum CompileError {
    VariableAlreadyDeclared(String),
    VariableUseInInitializer(String),
    UndefinedVariable {
        name: String,
        suggestion: Option<String>,
    },

    AssignmentToConstant(String),

    TooManyConstants,
    TooManyArguments,
    TooManyArrayElements,
    TooManyLocals,
    BreakOutsideLoop,
    ContinueOutsideLoop,
    ReturnOutsidFunction,
    TooManyUpvalues,
    ExpectedDeclarationAfterExport,
    ModuleParserErrors(Vec<ParserError>),
    ModuleLexerErrors(Vec<LexerError>),

    WrongArgumentCount {
        expected: i32,
        found: usize,
    },

    InvalidMemberAccess {
        name: String,
    },

    /// Incohérence interne du compilateur (ex. pile de boucles
    /// désynchronisée) — ne devrait jamais se produire si le compilateur
    /// est correct, mais on préfère un message d'erreur clair à un panic
    /// qui tue tout le processus si un futur bug en introduit une.
    InternalCompilerError(String),

    // ============================================================
    // MODULE / IMPORT / EXPORT
    // ============================================================
    DuplicateExport(String),

    CircularImport(String),

    ModuleNotFound(String),

    ModuleInvalidPath(String),

    ModuleReadError {
        path: String,
        message: String,
    },

    ModuleLexerError(LexerError),

    ModuleParserError(ParserError),

    ModuleRuntimeError {
        path: String,
        source: RuntimeError,
    },

    /// Erreur de compilation À L'INTÉRIEUR d'un module importé (avant même
    /// son exécution) — porte le chemin du module concerné, pour qu'un
    /// `CompileError::WithLocation` remonté depuis sa compilation indique
    /// clairement DANS QUEL FICHIER se trouve la ligne/colonne fautive.
    #[allow(clippy::enum_variant_names)]
    ModuleCompileError {
        path: String,
        source: Box<CompileError>,
    },

    ExportNotFound {
        module: String,
        name: String,
    },

    InvalidExport,
    InvalidImport,

    /// Enveloppe posée une seule fois, au tout dernier moment, autour de
    /// n'importe quelle autre variante — voir le commentaire d'en-tête.
    WithLocation {
        line: usize,
        column: usize,
        source: Box<CompileError>,
    },
    InvalidJump,
    JumpTooLarge,
    TooManyObjectFields,
    TooManyTupleElements,
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::WithLocation {
                line,
                column,
                source,
            } => {
                write!(f, "ligne {line}, colonne {column} : {source}")
            }

            CompileError::VariableAlreadyDeclared(e) => {
                write!(f, "Variable '{e}' déjà déclarée")
            }

            CompileError::VariableUseInInitializer(e) => {
                write!(f, "Variable '{e}' utilisée dans son propre initialiseur")
            }

            CompileError::UndefinedVariable { name, suggestion } => match suggestion {
                Some(suggestion) => write!(
                    f,
                    "Variable '{name}' non définie. Vouliez-vous dire '{suggestion}' ?"
                ),

                None => write!(f, "Variable '{name}' non définie"),
            },

            CompileError::AssignmentToConstant(e) => {
                write!(f, "Affectation à la constante '{e}'")
            }

            CompileError::TooManyConstants => {
                write!(f, "Trop de constantes dans ce fragment de code")
            }
            CompileError::TooManyArguments => {
                write!(f, "Trop d'arguments ou de paramètres")
            }
            CompileError::TooManyArrayElements => {
                write!(f, "Trop d'éléments dans le tableau")
            }
            CompileError::TooManyLocals => {
                write!(f, "Trop de variables locales dans cette fonction")
            }

            CompileError::BreakOutsideLoop => {
                write!(f, "'break' en dehors d'une boucle")
            }

            CompileError::ContinueOutsideLoop => {
                write!(f, "'continue' en dehors d'une boucle")
            }

            CompileError::ReturnOutsidFunction => {
                write!(f, "'return' en dehors d'une fonction")
            }

            CompileError::TooManyUpvalues => {
                write!(f, "Trop de variables capturées par cette closure")
            }

            CompileError::WrongArgumentCount { expected, found } => {
                write!(f, "{expected} argument(s) attendu(s), {found} fourni(s)")
            }

            CompileError::InvalidMemberAccess { name } => {
                write!(f, "Accès de membre invalide : '{name}'")
            }

            CompileError::InternalCompilerError(message) => {
                write!(f, "Erreur interne du compilateur : {message}")
            }

            // ====================================================
            // MODULE / IMPORT / EXPORT
            // ====================================================
            CompileError::DuplicateExport(name) => {
                write!(f, "L'export '{name}' est déjà déclaré")
            }

            CompileError::CircularImport(path) => {
                write!(f, "Import de module circulaire : {path}")
            }

            CompileError::ModuleNotFound(path) => {
                write!(f, "Module introuvable : {path}")
            }

            CompileError::ModuleInvalidPath(path) => {
                write!(f, "Chemin de module invalide : {path}")
            }

            CompileError::ModuleReadError { path, message } => {
                write!(f, "Impossible de lire le module '{path}' : {message}")
            }

            CompileError::ModuleLexerError(error) => {
                write!(f, "Erreur lexicale dans le module : {}", error.message)
            }

            CompileError::ModuleParserError(error) => {
                write!(f, "Erreur de syntaxe dans le module : {}", error.message)
            }

            CompileError::ModuleRuntimeError { path, source } => {
                write!(f, "Erreur d'exécution dans le module '{path}' : {source}")
            }

            CompileError::ModuleCompileError { path, source } => {
                write!(
                    f,
                    "Erreur de compilation dans le module '{path}' : {source}"
                )
            }

            CompileError::ExportNotFound { module, name } => {
                write!(f, "Le module '{module}' n'exporte pas '{name}'")
            }

            CompileError::InvalidExport => {
                write!(f, "Déclaration d'export invalide")
            }
            CompileError::InvalidImport => {
                write!(f, "Déclaration d'import invalide")
            }
            CompileError::ExpectedDeclarationAfterExport => {
                write!(f, "Déclaration attendue après 'export'")
            }
            CompileError::ModuleParserErrors(errors) => {
                for (index, error) in errors.iter().enumerate() {
                    if index > 0 {
                        writeln!(f)?;
                    }

                    write!(
                        f,
                        "Erreur de syntaxe dans le module à {}:{} : {}",
                        error.line, error.column, error.message
                    )?;
                }

                Ok(())
            }
            CompileError::ModuleLexerErrors(lexer_errors) => {
                for (index, error) in lexer_errors.iter().enumerate() {
                    if index > 0 {
                        writeln!(f)?;
                    }

                    write!(
                        f,
                        "Erreur lexicale dans le module à {}:{} : {}",
                        error.line, error.column, error.message
                    )?;
                }

                Ok(())
            }
            CompileError::InvalidJump => {
                write!(f, "Saut de bytecode invalide")
            }

            CompileError::JumpTooLarge => {
                write!(f, "Saut de bytecode trop grand")
            }
            CompileError::TooManyObjectFields => {
                write!(f, "Trop de champs dans cet objet")
            }
            CompileError::TooManyTupleElements => {
                write!(f, "Trop d'éléments dans le tuple")
            }
        }
    }
}

impl CompileError {
    /// Équivalent de `RuntimeError::to_diagnostic` côté compilateur : voir
    /// les commentaires de cette dernière pour la philosophie générale
    /// (position récupérée depuis `WithLocation`, expected/found/help
    /// uniquement là où l'information existe réellement).
    pub fn to_diagnostic(&self) -> crate::error::diagnostic::Diagnostic {
        use crate::error::diagnostic::Diagnostic;

        if let CompileError::WithLocation {
            line,
            column,
            source,
        } = self
        {
            let mut diagnostic = source.to_diagnostic();
            diagnostic.line = *line;
            diagnostic.column = *column;
            return diagnostic;
        }

        match self {
            CompileError::UndefinedVariable { name, suggestion } => {
                let diagnostic = Diagnostic::new(format!("variable '{name}' non définie"), 0, 0)
                    .with_len(name.len());

                match suggestion {
                    Some(suggestion) => {
                        diagnostic.with_help(format!("vouliez-vous dire '{suggestion}' ?"))
                    }
                    None => diagnostic,
                }
            }

            CompileError::VariableAlreadyDeclared(name) => Diagnostic::new(
                format!("la variable '{name}' est déjà déclarée dans cette portée"),
                0,
                0,
            )
            .with_len(name.len())
            .with_help(format!(
                "renommez l'une des deux déclarations, ou retirez `let`/`const` \
                 si vous vouliez réaffecter '{name}'."
            )),

            CompileError::VariableUseInInitializer(name) => Diagnostic::new(
                format!("'{name}' est utilisée dans son propre initialiseur"),
                0,
                0,
            )
            .with_len(name.len())
            .with_help(format!(
                "'{name}' n'existe pas encore au moment où son initialiseur s'exécute ; \
                 utilisez un autre nom pour la valeur dont vous avez besoin."
            )),

            CompileError::AssignmentToConstant(name) => Diagnostic::new(
                format!("impossible de réaffecter la constante '{name}'"),
                0,
                0,
            )
            .with_len(name.len())
            .with_expected("une variable déclarée avec 'let'")
            .with_found("une variable déclarée avec 'const'")
            .with_help(format!("déclarez '{name}' avec `let` au lieu de `const` si elle doit changer.")),

            CompileError::WrongArgumentCount { expected, found } => {
                Diagnostic::new("nombre d'arguments incorrect", 0, 0)
                    .with_expected(format!("{expected} argument(s)"))
                    .with_found(format!("{found} argument(s)"))
            }

            CompileError::InvalidMemberAccess { name } => {
                Diagnostic::new(format!("accès de membre invalide : '{name}'"), 0, 0)
                    .with_len(name.len())
            }

            CompileError::BreakOutsideLoop => {
                Diagnostic::new("'break' en dehors d'une boucle", 0, 0)
                    .with_len(5)
                    .with_help("'break' ne peut apparaître qu'à l'intérieur d'un 'while' ou d'un 'for'.")
            }

            CompileError::ContinueOutsideLoop => {
                Diagnostic::new("'continue' en dehors d'une boucle", 0, 0)
                    .with_len(8)
                    .with_help("'continue' ne peut apparaître qu'à l'intérieur d'un 'while' ou d'un 'for'.")
            }

            CompileError::ReturnOutsidFunction => {
                Diagnostic::new("'return' en dehors d'une fonction", 0, 0)
                    .with_len(6)
                    .with_help("'return' ne peut apparaître qu'à l'intérieur d'un 'func'.")
            }

            CompileError::TooManyUpvalues => Diagnostic::new(
                "trop de variables capturées par cette closure",
                0,
                0,
            )
            .with_help("réduisez le nombre de variables externes utilisées dans cette fonction imbriquée."),

            CompileError::ModuleNotFound(path) => {
                Diagnostic::new(format!("module introuvable : '{path}'"), 0, 0)
                    .with_len(path.len())
                    .with_help("vérifiez le chemin du module et son extension ('.ks').")
            }

            CompileError::ExportNotFound { module, name } => Diagnostic::new(
                format!("le module '{module}' n'exporte pas '{name}'"),
                0,
                0,
            )
            .with_len(name.len()),

            CompileError::CircularImport(path) => {
                Diagnostic::new(format!("import de module circulaire : '{path}'"), 0, 0)
                    .with_help("un module ne peut pas s'importer lui-même, directement ou indirectement.")
            }

            // Variantes sans donnée exploitable pour expected/found/help :
            // on garde un titre honnête (dérivé de Display) plutôt que
            // d'inventer une information qu'on n'a pas.
            other => Diagnostic::new(other.to_string(), 0, 0),
        }
    }
}

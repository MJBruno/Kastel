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
        }
    }
}

use std::fmt;

use crate::error::compile_error::CompileError;
use crate::error::lex_error::LexerError;
use crate::error::parse_error::ParserError;
use crate::error::runtime_error::RuntimeError;

/// Erreur unifiée regroupant les quatre phases du pipeline d'exécution
/// (lexer, parser, compilateur, VM).
///
/// Chaque phase a son propre type d'erreur (et le lexer/parser retournent
/// même un `Vec<...>` puisqu'ils collectent plusieurs erreurs avant
/// d'abandonner). `KastelError` sert uniquement de point de convergence pour
/// que `execute()` puisse enchaîner les `?` sans conversion manuelle à
/// chaque étape.
#[derive(Debug)]
pub enum KastelError {
    Lexer(Vec<LexerError>),
    Parser(Vec<ParserError>),
    Compile(CompileError),
    Runtime(RuntimeError),
}

impl From<Vec<LexerError>> for KastelError {
    fn from(errors: Vec<LexerError>) -> Self {
        KastelError::Lexer(errors)
    }
}

impl From<Vec<ParserError>> for KastelError {
    fn from(errors: Vec<ParserError>) -> Self {
        KastelError::Parser(errors)
    }
}

impl From<CompileError> for KastelError {
    fn from(error: CompileError) -> Self {
        KastelError::Compile(error)
    }
}

impl From<RuntimeError> for KastelError {
    fn from(error: RuntimeError) -> Self {
        KastelError::Runtime(error)
    }
}

impl fmt::Display for KastelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KastelError::Lexer(errors) => {
                writeln!(f, "Erreur(s) lexicale(s) :")?;

                for error in errors {
                    writeln!(
                        f,
                        "  - ligne {}, colonne {} : {}",
                        error.line, error.column, error.message
                    )?;
                }

                Ok(())
            }

            KastelError::Parser(errors) => {
                writeln!(f, "Erreur(s) de parsing :")?;

                for error in errors {
                    writeln!(
                        f,
                        "  - ligne {}, colonne {} : {}",
                        error.line, error.column, error.message
                    )?;
                }

                Ok(())
            }

            KastelError::Compile(error) => write!(f, "Erreur de compilation : {error}"),

            KastelError::Runtime(error) => write!(f, "Erreur d'exécution : {error}"),
        }
    }
}

impl std::error::Error for KastelError {}

impl KastelError {
    /// Rend l'erreur en un ou plusieurs encarts façon rustc (voir
    /// `error::diagnostic::Diagnostic`), en s'appuyant sur le texte source
    /// pour afficher l'extrait de code fautif. `file` est le nom affiché
    /// après `-->` (chemin du fichier exécuté, ou `<repl>` pour une
    /// session interactive).
    ///
    /// C'est la fonction à utiliser pour afficher une erreur à
    /// l'utilisateur (`Application::run`, REPL) ; le `Display` ci-dessus
    /// reste le format plat "ligne X, colonne Y : message", conservé pour
    /// le `Debug`/logs et pour ne rien casser côté `impl Error`.
    pub fn render(&self, source: &str, file: &str) -> String {
        use crate::error::diagnostic::Diagnostic;

        match self {
            // On ne garde que la PREMIÈRE erreur : le lexer/parser
            // continue après une erreur pour tenter d'en repérer d'autres
            // (récupération d'erreur), mais ces erreurs "en cascade" sont
            // presque toujours de simples conséquences de la première et
            // ne font que noyer la vraie cause sous du bruit — voir aussi
            // le même choix pour les erreurs de module
            // (`CompileError::ModuleParserErrors`/`ModuleLexerErrors`).
            KastelError::Lexer(errors) => match errors.first() {
                Some(error) => Diagnostic::new(error.message.clone(), error.line, error.column)
                    .render(source, file),
                None => String::new(),
            },

            KastelError::Parser(errors) => match errors.first() {
                Some(error) => Diagnostic::new(error.message.clone(), error.line, error.column)
                    .render(source, file),
                None => String::new(),
            },

            KastelError::Compile(error) => error.to_diagnostic().render(source, file),

            KastelError::Runtime(error) => error.to_diagnostic().render(source, file),
        }
    }
}

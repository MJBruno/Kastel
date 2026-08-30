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
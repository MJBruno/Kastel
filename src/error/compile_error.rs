use crate::error::lex_error::LexerError;
use crate::error::parse_error::ParserError;
use crate::error::runtime_error::RuntimeError;

// ================================================================
// COMPILE_ERROR
// ================================================================

#[derive(Debug)]
#[allow(dead_code)]
pub enum CompileError {
    VariableAlreadyDeclared(String),
    VariableUseInInitializer(String),
    UndefinedVariable(String),

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

    WrongArgumentCount { expected: i32, found: usize },

    InvalidMemberAccess { name: String },

    // ============================================================
    // MODULE / IMPORT / EXPORT
    // ============================================================
    DuplicateExport(String),

    CircularImport(String),

    ModuleNotFound(String),

    ModuleInvalidPath(String),

    ModuleReadError { path: String, message: String },

    ModuleLexerError(LexerError),

    ModuleParserError(ParserError),

    ModuleRuntimeError(RuntimeError),

    ExportNotFound { module: String, name: String },

    InvalidExport,
    
    InvalidImport,
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::VariableAlreadyDeclared(e) => {
                write!(f, "Variable {e} already declared")
            }

            CompileError::VariableUseInInitializer(e) => {
                write!(f, "Variable {e} not initialized")
            }

            CompileError::UndefinedVariable(e) => {
                write!(f, "Variable {e} not defined")
            }

            CompileError::AssignmentToConstant(e) => {
                write!(f, "Assignment to constant variable '{e}'")
            }

            CompileError::TooManyConstants => {
                write!(f, "Too many constants")
            }
            CompileError::TooManyArguments => {
                write!(f, "Too many arguments")
            }
            CompileError::TooManyArrayElements => {
                write!(f, "Too many array elements")
            }
            CompileError::TooManyLocals => {
                write!(f, "Too many local variables")
            }

            CompileError::BreakOutsideLoop => {
                write!(f, "Break outside loop")
            }

            CompileError::ContinueOutsideLoop => {
                write!(f, "Continue outside loop")
            }

            CompileError::ReturnOutsidFunction => {
                write!(f, "Return outside function")
            }

            CompileError::TooManyUpvalues => {
                write!(f, "Too many upvalues")
            }

            CompileError::WrongArgumentCount { expected, found } => {
                write!(f, "Expected {expected} arguments but found {found}.")
            }

            CompileError::InvalidMemberAccess { name } => {
                write!(f, "Invalid member access '{name}'")
            }

            // ====================================================
            // MODULE / IMPORT / EXPORT
            // ====================================================
            CompileError::DuplicateExport(name) => {
                write!(f, "Export '{name}' is already declared")
            }

            CompileError::CircularImport(path) => {
                write!(f, "Circular module import: {path}")
            }

            CompileError::ModuleNotFound(path) => {
                write!(f, "Module not found: {path}")
            }

            CompileError::ModuleInvalidPath(path) => {
                write!(f, "Invalid module path: {path}")
            }

            CompileError::ModuleReadError { path, message } => {
                write!(f, "Unable to read module '{path}': {message}")
            }

            CompileError::ModuleLexerError(message) => {
                write!(f, "Lexer error in module: {}", message.message)
            }

            CompileError::ModuleParserError(error) => {
                write!(f, "Parser error in module: {}", error.message)
            }

            CompileError::ModuleRuntimeError(message) => {
                write!(f, "Runtime error in module: {message}")
            }

            CompileError::ExportNotFound { module, name } => {
                write!(f, "Module '{module}' does not export '{name}'")
            }

            CompileError::InvalidExport => {
                write!(f, "Invalid export declaration")
            }
            CompileError::InvalidImport => {
                write!(f, "Invalid import declaration")
            }
            CompileError::ExpectedDeclarationAfterExport => {
                write!(f, "Expected declaration after export")
            }
            CompileError::ModuleParserErrors(errors) => {
                for (index, error) in errors.iter().enumerate() {
                    if index > 0 {
                        writeln!(f)?;
                    }

                    write!(
                        f,
                        "Parser error in module at {}:{}: {}",
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
                        "Parser error in module at {}:{}: {}",
                        error.line, error.column, error.message
                    )?;
                }

                Ok(())
            }
        }
    }
}

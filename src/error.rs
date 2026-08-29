// ================================================================
// LEXER_ERROR
// ================================================================

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LexerError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

// ================================================================
// PARSE_ERROR
// ================================================================

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ParserError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

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

// ================================================================
// VM_ERROR
// ================================================================

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum VMError {
    StackUnderflow,
    InvalidConstantIndex(u16),
    TypeError(String),
    DivisionByZero,
    InvalidInstruction(String),
}

impl std::fmt::Display for VMError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VMError::StackUnderflow => {
                write!(formatter, "stack underflow")
            }

            VMError::InvalidConstantIndex(index) => {
                write!(formatter, "invalid constant index: {index}")
            }

            VMError::TypeError(message) => {
                write!(formatter, "type error: {message}")
            }

            VMError::DivisionByZero => {
                write!(formatter, "division by zero")
            }

            VMError::InvalidInstruction(message) => {
                write!(formatter, "invalid instruction: {message}")
            }
        }
    }
}

// ================================================================
// RUNTIME_ERROR
// ================================================================

#[allow(dead_code)]
#[derive(Debug)]
pub enum RuntimeError {
    TypeError,
    DivisionByZero,

    WrongArgumentCount { expected: usize, found: usize },

    NotCallable,
    InvalidFunction,
    NativeError,
    IndexOutOfBounds,

    ArrayEmpty,

    ArrayIndexNotInteger,

    ArrayIndexOutOfBounds { index: usize, length: usize },
    ModuleError(String),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::TypeError => {
                write!(f, "Operand must be numbers.")
            }

            RuntimeError::ArrayEmpty => {
                write!(f, "Array empty")
            }

            RuntimeError::DivisionByZero => {
                write!(f, "Division by zero.")
            }

            RuntimeError::WrongArgumentCount { expected, found } => {
                write!(f, "Expected {expected} arguments but found {found}.")
            }

            RuntimeError::NotCallable => {
                write!(f, "Value is not callable.")
            }

            RuntimeError::InvalidFunction => {
                write!(f, "Invalid function.")
            }

            RuntimeError::NativeError => {
                write!(f, "Native function error.")
            }

            RuntimeError::ArrayIndexNotInteger => {
                write!(f, "Array index must be an integer.")
            }

            RuntimeError::ArrayIndexOutOfBounds { index, length } => {
                write!(f, "Array index {index} out of bounds for length {length}.")
            }

            RuntimeError::IndexOutOfBounds => {
                write!(f, "Array index out of bounds")
            }
            RuntimeError::ModuleError(message) => {
                write!(f, "Module error: {message}")
            }
        }
    }
}

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
pub enum CompileError {
    VariableAlreadyDeclared(String),
    VariableUseInInitializer(String),
    UndefinedVariable(String),

    AssignmentToConstant(String),

    TooManyConstants,
    TooManyLocals,
    BreakOutsideLoop,
    ContinueOutsideLoop,
    ReturnOutsidFunction,
    TooManyUpvalues,
    WrongArgumentCount { expected: i32, found: usize },
    InvalidMemberAccess { name: String },
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
pub enum RuntimeError {
    TypeError,
    DivisionByZero,

    WrongArgumentCount { expected: usize, found: usize },

    NotCallable,
    InvalidFunction,
    NativeError,
    IndexOutOfBounds,

    ArrayIndexNotInteger,
    ArrayIndexOutOfBounds { index: usize, length: usize },
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::TypeError => {
                write!(f, "Operand must be numbers.")
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
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LexerError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ParserError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug)]
pub enum CompileError {
    VariableAlreadyDeclared(String),
    VariableUseInInitializer(String),
    UndefinedVariable(String),
    TooManyConstants,
    TooManyLocals,
    BreakOutsideLoop,
    ContinueOutsideLoop,
    ReturnOutsidFunction,
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::VariableAlreadyDeclared(e) => write!(f, "Variable {e} not declare"),
            CompileError::VariableUseInInitializer(e) => write!(f, "Variable {e} not initialize"),
            CompileError::UndefinedVariable(e) => write!(f, "Variable {e} not defined"),
            CompileError::TooManyConstants => write!(f, "Too many constant"),
            CompileError::TooManyLocals => write!(f, "Too many local"),
            CompileError::BreakOutsideLoop => write!(f, "Break outside loop"),
            CompileError::ContinueOutsideLoop => write!(f, "Continue outside loop"),
            CompileError::ReturnOutsidFunction => write!(f, "Return outside function"),
        }
    }
}

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
            VMError::StackUnderflow => write!(formatter, "stack underflow"),

            VMError::InvalidConstantIndex(index) => {
                write!(formatter, "invalid constant index: {index}")
            }

            VMError::TypeError(message) => write!(formatter, "type error: {message}"),

            VMError::DivisionByZero => write!(formatter, "division by zero"),

            VMError::InvalidInstruction(message) => {
                write!(formatter, "invalid instruction: {message}")
            }
        }
    }
}



pub enum RuntimeError {
    TypeError,
    DivisionByZero,
    WrongArgumentCount { expected: usize, found: usize },
    NotCallable,
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
                write!(
                    f,
                    "Mauvais liste d'argument au lieu de {expected} or {found}."
                )
            }
            RuntimeError::NotCallable => {
                write!(f, "Cette expression ne peut pas être appeler")
            }
        }
    }
}

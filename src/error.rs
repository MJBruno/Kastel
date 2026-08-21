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
}

impl Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::VariableAlreadyDeclared(e) => write!(f, "Variable {e} déja déclarer"),
            CompileError::VariableUseInInitializer(e) => write!(f, "Variable {e} non initialisé"),
            CompileError::UndefinedVariable(e) => write!(f, "Variable {e} non définie"),
            CompileError::TooManyConstants => write!(f, "Trop de constant"),
            CompileError::TooManyLocals => write!(f, "Trop de local"),
            CompileError::BreakOutsideLoop => write!(f, "Break hors boucle"),
            CompileError::ContinueOutsideLoop => write!(f, "Continue hors boucle"),
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

use std::fmt::Display;

pub enum RuntimeError {
    TypeError,
    DivisionByZero,
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::TypeError => {
                write!(f, "Operand must be numbers.")
            }
            RuntimeError::DivisionByZero => {
                write!(f, "Division by zero.")
            }
        }
    }
}

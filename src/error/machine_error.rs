

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
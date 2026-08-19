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

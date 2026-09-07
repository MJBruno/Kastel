// ================================================================
// RUNTIME_ERROR
// ================================================================

 
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeError {
    TypeError,
    DivisionByZero,

    WrongArgumentCount {
        expected: usize,
        found: usize,
    },

    NotCallable,
    InvalidFunction,
    NativeError,

    IndexOutOfBounds,
    ArrayIndexNotInteger,
    ArrayIndexOutOfBounds {
        index: usize,
        length: usize,
    },

    NotIndexable,
    NotObject,

    ModuleError(String),

    ObjectFieldNotFound {
        name: String,
        suggestion: Option<String>,
    },

    NotIterable,
    IteratorExhausted,
    InvalidShiftAmount,

    WithLocation {
        line: usize,
        column: usize,
        source: Box<RuntimeError>,
    },

    StackUnderflow,
    InvalidOpcode(u8),
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
                write!(
                    f,
                    "Array index {index} out of bounds for length {length}."
                )
            }

            RuntimeError::IndexOutOfBounds => {
                write!(f, "Array index out of bounds")
            }

            RuntimeError::ModuleError(message) => {
                write!(f, "Module error: {message}")
            }

            RuntimeError::ObjectFieldNotFound { name, suggestion } => {
                match suggestion {
                    Some(suggestion) => write!(
                        f,
                        "Champ '{name}' introuvable sur l'objet. Vouliez-vous dire '{suggestion}' ?"
                    ),

                    None => write!(
                        f,
                        "Champ '{name}' introuvable sur l'objet."
                    ),
                }
            }

            RuntimeError::NotIterable => {
                write!(
                    f,
                    "Cette valeur n'est pas itérable (utilisable dans un 'for..in')."
                )
            }

            RuntimeError::IteratorExhausted => {
                write!(f, "Itérateur déjà épuisé.")
            }

            RuntimeError::InvalidShiftAmount => {
                write!(
                    f,
                    "Décalage invalide : doit être compris entre 0 et 63."
                )
            }

            RuntimeError::WithLocation {
                line,
                column,
                source,
            } => {
                write!(f, "ligne {line}, colonne {column} : {source}")
            }

            RuntimeError::NotIndexable => {
                write!(f, "Value is not indexable.")
            }

            RuntimeError::NotObject => {
                write!(f, "Value is not an object.")
            }

            RuntimeError::StackUnderflow => {
                write!(f, "VM stack underflow.")
            }

            RuntimeError::InvalidOpcode(opcode) => {
                write!(f, "Invalid bytecode opcode: {opcode}.")
            }
        }
    }
}
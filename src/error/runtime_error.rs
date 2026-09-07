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

    /// `suggestion` : nom de champ existant le plus proche, si un match
    /// assez proche a été trouvé (voir error::suggest::closest_match).
    ObjectFieldNotFound {
        name: String,
        suggestion: Option<String>,
    },

    NotIterable,
    IteratorExhausted,
    InvalidShiftAmount,

    /// Enveloppe posée une seule fois, au moment où l'erreur s'échappe de
    /// `VirtualMachine::run()`, pour lui attacher la position source de
    /// l'instruction qui a échoué (voir `Chunk::position_at`).
    WithLocation {
        line: usize,
        column: usize,
        source: Box<RuntimeError>,
    },
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
            RuntimeError::ObjectFieldNotFound { name, suggestion } => match suggestion {
                Some(suggestion) => write!(
                    f,
                    "Champ '{name}' introuvable sur l'objet. Vouliez-vous dire '{suggestion}' ?"
                ),

                None => write!(f, "Champ '{name}' introuvable sur l'objet."),
            },
            RuntimeError::NotIterable => {
                write!(f, "Cette valeur n'est pas itérable (utilisable dans un 'for..in').")
            }
            RuntimeError::IteratorExhausted => {
                write!(f, "Itérateur déjà épuisé.")
            }
            RuntimeError::InvalidShiftAmount => {
                write!(f, "Décalage invalide : doit être compris entre 0 et 63.")
            }
            RuntimeError::WithLocation { line, column, source } => {
                write!(f, "ligne {line}, colonne {column} : {source}")
            }
        }
    }
}
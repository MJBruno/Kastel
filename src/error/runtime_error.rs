use crate::runtime::value::{NumericOp, Value};

// ================================================================
// RUNTIME_ERROR
// ================================================================

#[derive(Debug, Clone)]
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

    /// Tentative de mutation d'une valeur immuable (ex. `tuple[0] = x`).
    /// Porte le nom du type concerné pour le message d'erreur.
    ImmutableValue(&'static str),

    ModuleError(String),

    ObjectFieldNotFound {
        name: String,
        suggestion: Option<String>,
    },

    NotIterable,
    IteratorExhausted,
    InvalidShiftAmount,

    /// Erreur de type enrichie pour une opération arithmétique binaire
    /// (`+ - * / %`) entre deux types incompatibles — porte l'opération et
    /// les deux types réellement rencontrés, pour un diagnostic
    /// expected/found précis. `TypeError` (générique) reste utilisé pour
    /// tous les autres cas (accès, appels, itération...) afin de ne pas
    /// avoir à faire remonter cette information partout dans la VM.
    NumericTypeError {
        operation: NumericOp,
        expected: &'static str,
        found: &'static str,
    },

    WithLocation {
        line: usize,
        column: usize,
        source: Box<RuntimeError>,
    },

    StackUnderflow,
    InvalidOpcode(u8),

    // ============================================================
    // EXCEPTION KASTEL
    // ============================================================

    /*
     * Exception explicitement lancée par :
     *
     *     throw value;
     *
     * La valeur peut être n'importe quelle Value Kastel :
     *
     *     throw "boom";
     *     throw 42;
     *     throw [1, 2, 3];
     *     throw { message: "boom" };
     */
    Thrown(Value),

    InterfaceMethodMissing {
        interface: String,
        method: String,
    },

    InterfaceMethodArityMismatch {
        interface: String,
        method: String,
        expected: usize,
        found: usize,
    },
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
                write!(
                    f,
                    "Cette valeur n'est pas itérable (utilisable dans un 'for..in')."
                )
            }

            RuntimeError::IteratorExhausted => {
                write!(f, "Itérateur déjà épuisé.")
            }

            RuntimeError::InvalidShiftAmount => {
                write!(f, "Décalage invalide : doit être compris entre 0 et 63.")
            }

            RuntimeError::NumericTypeError {
                operation,
                expected,
                found,
            } => {
                write!(
                    f,
                    "Impossible de {} une valeur '{expected}' et une valeur '{found}' (opérateur '{}').",
                    operation.verb(),
                    operation.symbol()
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

            RuntimeError::ImmutableValue(type_name) => {
                write!(
                    f,
                    "Impossible de modifier une valeur de type '{type_name}' : elle est immuable."
                )
            }

            RuntimeError::StackUnderflow => {
                write!(f, "VM stack underflow.")
            }

            RuntimeError::InvalidOpcode(opcode) => {
                write!(f, "Invalid bytecode opcode: {opcode}.")
            }

            RuntimeError::Thrown(value) => {
                write!(f, "Uncaught exception: {value:?}")
            }

            RuntimeError::InterfaceMethodMissing { interface, method } => {
                write!(f, "Interface '{}' requires method '{}'.", interface, method)
            }

            RuntimeError::InterfaceMethodArityMismatch {
                interface,
                method,
                expected,
                found,
            } => {
                write!(
                    f,
                    "Method '{}' from interface '{}' expects {} arguments but found {}.",
                    method, interface, expected, found
                )
            }
        }
    }
}

impl RuntimeError {
    /// Construit le `Diagnostic` (position + titre + éventuels
    /// expected/found/help) à afficher pour cette erreur.
    ///
    /// La position vient de la variante `WithLocation` la plus externe, si
    /// présente (c'est toujours le cas pour une erreur qui a atteint la
    /// frontière `Application::run` / REPL — voir `error/mod.rs`). Pour le
    /// reste (titre, expected/found, help), seules les variantes qui
    /// portent réellement l'information nécessaire la fournissent ; les
    /// autres se contentent d'un titre basé sur leur `Display`, ce qui
    /// reste honnête plutôt que d'inventer un expected/found fictif.
    pub fn to_diagnostic(&self) -> crate::error::diagnostic::Diagnostic {
        use crate::error::diagnostic::Diagnostic;

        if let RuntimeError::WithLocation {
            line,
            column,
            source,
        } = self
        {
            let mut diagnostic = source.to_diagnostic();

            // Même précaution que côté `CompileError::to_diagnostic` : ne
            // pas écraser la position d'une erreur qui pointe déjà vers un
            // autre fichier (module importé).
            if diagnostic.source_override.is_none() {
                diagnostic.line = *line;
                diagnostic.column = *column;
            }

            return diagnostic;
        }

        match self {
            RuntimeError::NumericTypeError {
                operation,
                expected,
                found,
            } => Diagnostic::new(
                format!("impossible de {} un(e) '{expected}' et un(e) '{found}'", operation.verb()),
                0,
                0,
            )
            .with_expected(*expected)
            .with_found(*found)
            .with_help(format!(
                "convertissez l'un des deux opérandes pour qu'ils soient du même type, \
                 par exemple avec str(...) pour obtenir une chaîne, ou int(...) / float(...) \
                 pour obtenir un nombre (opérateur '{}').",
                operation.symbol()
            )),

            RuntimeError::DivisionByZero => Diagnostic::new("division par zéro", 0, 0).with_help(
                "vérifiez que le diviseur n'est jamais nul avant l'opération \
                 (par exemple avec `if diviseur != 0 { ... }`).",
            ),

            RuntimeError::WrongArgumentCount { expected, found } => {
                Diagnostic::new("nombre d'arguments incorrect", 0, 0)
                    .with_expected(format!("{expected} argument(s)"))
                    .with_found(format!("{found} argument(s)"))
            }

            RuntimeError::ArrayIndexOutOfBounds { index, length } => {
                Diagnostic::new("index de tableau hors limites", 0, 0)
                    .with_expected(if *length == 0 {
                        "un tableau non vide".to_string()
                    } else {
                        format!("un index entre 0 et {}", *length - 1)
                    })
                    .with_found(format!("{index}"))
            }

            RuntimeError::ArrayIndexNotInteger => Diagnostic::new(
                "l'index d'un tableau doit être un entier",
                0,
                0,
            )
            .with_expected("integer"),

            RuntimeError::ObjectFieldNotFound { name, suggestion } => {
                let diagnostic = Diagnostic::new(
                    format!("le champ '{name}' n'existe pas sur cet objet"),
                    0,
                    0,
                )
                .with_len(name.len());

                match suggestion {
                    Some(suggestion) => {
                        diagnostic.with_help(format!("vouliez-vous dire '{suggestion}' ?"))
                    }
                    None => diagnostic,
                }
            }

            RuntimeError::ImmutableValue(type_name) => Diagnostic::new(
                format!("impossible de modifier une valeur de type '{type_name}' : elle est immuable"),
                0,
                0,
            )
            .with_help(format!(
                "les valeurs de type '{type_name}' ne peuvent pas être modifiées en place ; \
                 créez-en une nouvelle à la place."
            )),

            RuntimeError::NotCallable => {
                Diagnostic::new("cette valeur n'est pas appelable", 0, 0)
                    .with_expected("function")
                    .with_help("vérifiez que vous appelez bien une fonction, une closure ou une méthode.")
            }

            RuntimeError::NotIterable => Diagnostic::new(
                "cette valeur n'est pas itérable (utilisable dans un 'for..in')",
                0,
                0,
            )
            .with_expected("array, tuple, range, dict ou iterator"),

            RuntimeError::NotIndexable => Diagnostic::new("cette valeur n'est pas indexable", 0, 0)
                .with_expected("array, tuple, string ou object"),

            RuntimeError::NotObject => {
                Diagnostic::new("cette valeur n'est pas un objet", 0, 0).with_expected("object")
            }

            RuntimeError::IteratorExhausted => {
                Diagnostic::new("cet itérateur est déjà épuisé", 0, 0).with_help(
                    "vérifiez `has_next()` avant d'appeler `next()`, ou recréez l'itérateur.",
                )
            }

            RuntimeError::InvalidShiftAmount => {
                Diagnostic::new("décalage de bits invalide", 0, 0)
                    .with_expected("un entier entre 0 et 63")
            }

            RuntimeError::ModuleError(message) => {
                Diagnostic::new(format!("erreur de module : {message}"), 0, 0)
            }

            RuntimeError::Thrown(value) => {
                Diagnostic::new(format!("exception non interceptée : {value}"), 0, 0).with_help(
                    "encadrez le code qui peut lever cette exception avec `try { ... } catch (e) { ... }`.",
                )
            }

            RuntimeError::InterfaceMethodMissing { interface, method } => Diagnostic::new(
                format!("il manque la méthode '{method}' requise par l'interface '{interface}'"),
                0,
                0,
            )
            .with_help(format!("ajoutez `func {method}(...) {{ ... }}` à la classe.")),

            RuntimeError::InterfaceMethodArityMismatch {
                interface,
                method,
                expected,
                found,
            } => Diagnostic::new(
                format!(
                    "la méthode '{method}' de l'interface '{interface}' n'a pas la bonne arité"
                ),
                0,
                0,
            )
            .with_expected(format!("{expected} paramètre(s)"))
            .with_found(format!("{found} paramètre(s)")),

            // Variantes sans donnée exploitable pour expected/found/help :
            // on garde un titre honnête (dérivé de Display) plutôt que
            // d'inventer une information qu'on n'a pas.
            other => Diagnostic::new(other.to_string(), 0, 0),
        }
    }
}

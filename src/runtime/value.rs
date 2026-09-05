use crate::error::runtime_error::RuntimeError;
use crate::runtime::gc_handle::Gc;
use crate::runtime::native::NativeFn;
use crate::runtime::object::Object;

pub enum NumericOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

#[allow(dead_code)]
pub enum ComparisonOp {
    Equal,
    Greater,
    Less,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code, unpredictable_function_pointer_comparisons)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Boolean(bool),

    /// Pointeur de fonction native : `Copy`, aucune allocation, ne peut
    /// jamais former de cycle — reste un Value primitif, hors du système
    /// `Object`/`Gc` (voir object.rs pour la justification complète).
    NativeFunction(NativeFn),

    /// Intervalle numérique léger et RÉUTILISABLE, produit par range().
    /// Aucune allocation sur le tas (juste 3 f64) — c'est ce qui rend
    /// range() paresseux. Reste hors du système `Object`/`Gc` pour la
    /// même raison que NativeFunction : l'y faire entrer ajouterait une
    /// allocation à chaque appel de range(), ce qui va à l'encontre du
    /// but recherché.
    Range { start: f64, stop: f64, step: f64 },

    /// TOUT ce qui est alloué sur le tas et suivi par le collecteur de
    /// cycles passe par cette seule variante : chaînes, tableaux, objets
    /// dynamiques, fonctions, closures, itérateurs à état, modules — voir
    /// object.rs pour le détail de chaque variante d'`Object`. Le GC n'a
    /// plus qu'UN SEUL registre à parcourir (voir gc.rs), au lieu d'un
    /// par type comme avant cette refonte.
    Object(Gc<Object>),

    Nil,
}

#[allow(dead_code)]
impl Value {
    // ============================================================
    // CONSTRUCTEURS DE COMMODITÉ
    // ============================================================
    //
    // Évitent d'avoir à écrire `Value::Object(Gc::new(Object::String(...)))`
    // en toutes lettres à chaque site d'appel — c'est LE point de passage
    // obligé pour créer chacun de ces types, exactement comme
    // `objet::new_closure`/`new_upvalue` l'étaient déjà avant cette
    // refonte pour Closure/ObjUpvalue.

    pub fn new_string(value: String) -> Self {
        let handle = Gc::new(Object::String(value));

        crate::runtime::gc::register_object(&handle);

        Value::Object(handle)
    }

    pub fn new_array(elements: Vec<Value>) -> Self {
        let handle = Gc::new(Object::Array(elements));

        crate::runtime::gc::register_object(&handle);

        Value::Object(handle)
    }

    /// Extrait le `Gc<Object>` sous-jacent si cette valeur en est un.
    /// Sert de point d'entrée générique pour tout code qui a besoin de
    /// travailler avec la poignée elle-même plutôt qu'un accès typé
    /// (ex. le marquage GC, ou le dispatch générique de propriétés).
    pub fn as_object(&self) -> Option<&Gc<Object>> {
        match self {
            Value::Object(handle) => Some(handle),
            _ => None,
        }
    }
}

#[allow(dead_code)]
impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(value) => {
                write!(f, "{value}")
            }

            Value::Float(value) => {
                // Toujours au moins un point décimal, même pour une valeur
                // entière (5.0, pas 5) : c'est ce qui permet de distinguer
                // visuellement un Float d'un Integer, comme repr() en
                // Python. NaN/Infinity gardent leur affichage naturel.
                if value.fract() == 0.0 && value.is_finite() {
                    write!(f, "{value:.1}")
                } else {
                    write!(f, "{value}")
                }
            }

            Value::Boolean(value) => {
                write!(f, "{value}")
            }

            Value::Nil => {
                write!(f, "nil")
            }

            Value::NativeFunction(function) => {
                write!(f, "<nativeFn '{:?}'>", function)
            }

            Value::Range { start, stop, step } => {
                if *step == 1.0 {
                    write!(f, "range({start}, {stop})")
                } else {
                    write!(f, "range({start}, {stop}, {step})")
                }
            }

            Value::Object(handle) => match &*handle.borrow() {
                Object::String(value) => write!(f, "{value}"),

                Object::Array(array) => {
                    write!(f, "[")?;

                    for (index, value) in array.iter().enumerate() {
                        if index > 0 {
                            write!(f, ", ")?;
                        }

                        write!(f, "{value}")?;
                    }

                    write!(f, "]")
                }

                Object::Dict(fields) => {
                    write!(f, "{{")?;

                    for (index, (key, value)) in fields.iter().enumerate() {
                        if index > 0 {
                            write!(f, ", ")?;
                        }

                        write!(f, "{key}: {value}")?;
                    }

                    write!(f, "}}")
                }

                Object::Function(function) => {
                    write!(f, "<fun '{}'>", function.name)
                }

                Object::Closure(closure) => {
                    write!(f, "<closure '{}'>", closure.function.name)
                }

                Object::Iterator(_) => {
                    write!(f, "<iterator>")
                }

                Object::Module(module) => {
                    write!(f, "<module '{}'>", module.name)
                }
            },
        }
    }
}

#[allow(dead_code)]
impl Value {
    // ============================================================
    // TRUTHINESS
    // ============================================================

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Nil => false,

            Value::Boolean(value) => *value,

            Value::Integer(value) => *value != 0,

            Value::Float(value) => *value != 0.0 && !value.is_nan(),

            Value::Object(handle) => match &*handle.borrow() {
                Object::String(value) => !value.is_empty(),
                _ => true,
            },

            _ => true,
        }
    }

    // ============================================================
    // NUMERIC OPERATIONS
    // ============================================================

    pub fn binary_numeric_op(a: Value, b: Value, op: NumericOp) -> Result<Value, RuntimeError> {
        match (a, b) {
            // Int op Int -> Int (sauf division, toujours "vraie division"
            // façon Python 3 : 7 / 2 == 3.5, pas 3 — Kastel n'a pas
            // d'opérateur de division entière séparé).
            (Value::Integer(a), Value::Integer(b)) => match op {
                NumericOp::Add => Ok(Value::Integer(a.wrapping_add(b))),
                NumericOp::Subtract => Ok(Value::Integer(a.wrapping_sub(b))),
                NumericOp::Multiply => Ok(Value::Integer(a.wrapping_mul(b))),

                NumericOp::Divide => {
                    if b == 0 {
                        return Err(RuntimeError::DivisionByZero);
                    }

                    Ok(Value::Float(a as f64 / b as f64))
                }

                NumericOp::Modulo => {
                    if b == 0 {
                        return Err(RuntimeError::DivisionByZero);
                    }

                    Ok(Value::Integer(a.wrapping_rem(b)))
                }
            },

            // Toute combinaison impliquant un Float est promue en Float.
            (Value::Integer(a), Value::Float(b)) => Self::number_op(a as f64, b, op),
            (Value::Float(a), Value::Integer(b)) => Self::number_op(a, b as f64, op),
            (Value::Float(a), Value::Float(b)) => Self::number_op(a, b, op),

            _ => Err(RuntimeError::TypeError),
        }
    }

    fn number_op(a: f64, b: f64, op: NumericOp) -> Result<Value, RuntimeError> {
        match op {
            NumericOp::Add => Ok(Value::Float(a + b)),

            NumericOp::Subtract => Ok(Value::Float(a - b)),

            NumericOp::Multiply => Ok(Value::Float(a * b)),

            NumericOp::Divide => {
                if b == 0.0 {
                    return Err(RuntimeError::DivisionByZero);
                }

                Ok(Value::Float(a / b))
            }

            NumericOp::Modulo => Ok(Value::Float(a % b)),
        }
    }

    pub fn negate_values(a: Value) -> Result<Value, RuntimeError> {
        match a {
            Value::Integer(a) => Ok(Value::Integer(a.wrapping_neg())),

            Value::Float(a) => Ok(Value::Float(-a)),

            _ => Err(RuntimeError::TypeError),
        }
    }

    // ============================================================
    // COMPARISON
    // ============================================================

    /// Compare deux valeurs numériques, Integer et Float mélangeables
    /// (5 < 5.5 doit fonctionner). Passe par f64 pour la comparaison
    /// inter-types — limite connue : au-delà de 2^53, deux i64 distincts
    /// peuvent devenir "égaux" une fois convertis en f64. Kastel n'a pas
    /// vocation à manipuler des entiers de cette taille pour l'instant.
    pub fn compare_numeric(a: Value, b: Value, op: ComparisonOp) -> Result<Value, RuntimeError> {
        match (a, b) {
            (Value::Integer(a), Value::Integer(b)) => {
                let result = match op {
                    ComparisonOp::Equal => a == b,
                    ComparisonOp::Greater => a > b,
                    ComparisonOp::Less => a < b,
                };

                Ok(Value::Boolean(result))
            }

            (Value::Integer(a), Value::Float(b)) => {
                Ok(Value::Boolean(Self::compare_number(a as f64, b, op)))
            }

            (Value::Float(a), Value::Integer(b)) => {
                Ok(Value::Boolean(Self::compare_number(a, b as f64, op)))
            }

            (Value::Float(a), Value::Float(b)) => {
                Ok(Value::Boolean(Self::compare_number(a, b, op)))
            }

            _ => Err(RuntimeError::TypeError),
        }
    }

    fn compare_number(a: f64, b: f64, op: ComparisonOp) -> bool {
        match op {
            ComparisonOp::Equal => a == b,
            ComparisonOp::Greater => a > b,
            ComparisonOp::Less => a < b,
        }
    }

    // ============================================================
    // EQUALITY
    // ============================================================

    pub fn equals(a: Value, b: Value) -> bool {
        match (a, b) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Integer(a), Value::Float(b)) => a as f64 == b,
            (Value::Float(a), Value::Integer(b)) => a == b as f64,
            (Value::Float(a), Value::Float(b)) => a == b,

            (Value::Boolean(a), Value::Boolean(b)) => a == b,

            // Seules les chaînes sont comparées PAR VALEUR ici, comme
            // avant cette refonte (tableaux/objets/closures n'étaient déjà
            // pas comparés structurellement par `==` : ce n'est pas une
            // régression, c'est la même limite qu'avant, juste préservée).
            (Value::Object(a), Value::Object(b)) => {
                match (&*a.borrow(), &*b.borrow()) {
                    (Object::String(a), Object::String(b)) => a == b,
                    _ => false,
                }
            }

            (Value::Nil, Value::Nil) => true,

            _ => false,
        }
    }
}
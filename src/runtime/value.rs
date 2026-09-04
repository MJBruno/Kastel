use crate::module::module::ModuleInstance;
use crate::runtime::closure::Closure;
use crate::runtime::iterator::IteratorState;
use crate::runtime::native::NativeFn;
use crate::{error::runtime_error::RuntimeError, runtime::function::Function};
 
 

use std::{cell::RefCell, fmt::Display, rc::Rc};

pub type ArrayRef = Rc<RefCell<Vec<Value>>>;

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
    String(String),

    Function(Rc<Function>),
    
    Closure(Rc<RefCell<Closure>>),

    NativeFunction(NativeFn),

    Array(ArrayRef),

    /// Objet dynamique : liste ordonnée de (clé, valeur), comme un tableau
    /// associatif à la JS. Ordonné (pas de HashMap) pour préserver l'ordre
    /// d'écriture du littéral lors de l'affichage.
    Object(Rc<RefCell<Vec<(String, Value)>>>),

    /// Intervalle numérique léger et RÉUTILISABLE, produit par range().
    /// Aucune allocation sur le tas (juste 3 f64) — c'est ce qui rend
    /// range() paresseux. Parcourir deux fois le même Range donne deux
    /// fois la séquence complète, comme en Python.
    Range { start: f64, stop: f64, step: f64 },

    /// Curseur d'itération À ÉTAT, à usage unique. Créé fraîchement à
    /// chaque `for..in` via `Value::to_iterator()` — jamais construit
    /// directement par du code utilisateur.
    Iterator(Rc<RefCell<IteratorState>>),

    Module(Rc<ModuleInstance>),

    Nil,
}

#[allow(dead_code)]
impl Display for Value {
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

            Value::String(value) => {
                write!(f, "{value}")
            }

            Value::Nil => {
                write!(f, "nil")
            }

            Value::Function(function) => {
                write!(f, "<fun '{}'>", function.name)
            }

            Value::Closure(closure) => {
                let closure = closure.borrow();

                write!(f, "<closure '{}'>", closure.function.name)
            }

            Value::NativeFunction(function) => {
                write!(f, "<nativeFn '{:?}'>", function)
            }

            Value::Array(array) => {
                let array = array.borrow();

                write!(f, "[")?;

                for (index, value) in array.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{value}")?;
                }

                write!(f, "]")
            }

            Value::Object(fields) => {
                let fields = fields.borrow();

                write!(f, "{{")?;

                for (index, (key, value)) in fields.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{key}: {value}")?;
                }

                write!(f, "}}")
            }

            Value::Range { start, stop, step } => {
                if *step == 1.0 {
                    write!(f, "range({start}, {stop})")
                } else {
                    write!(f, "range({start}, {stop}, {step})")
                }
            }

            Value::Iterator(_) => {
                write!(f, "<iterator>")
            }

            Value::Module(module) => {
                write!(f, "<module '{}'>", module.name)
            }
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

            Value::String(value) => !value.is_empty(),

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

            (Value::String(a), Value::String(b)) => a == b,

            (Value::Nil, Value::Nil) => true,

            _ => false,
        }
    }
}
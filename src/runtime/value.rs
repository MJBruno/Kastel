use crate::module::module::ModuleInstance;
use crate::runtime::closure::Closure;
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
    Number(f64),
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

    Module(Rc<ModuleInstance>),

    Nil,
}

#[allow(dead_code)]
impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(value) => {
                write!(f, "{value}")
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

                write!(f, "{{ ")?;

                for (index, (key, value)) in fields.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{key}: {value}")?;
                }

                write!(f, " }}")
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

            Value::Number(value) => *value != 0.0 && !value.is_nan(),

            Value::String(value) => !value.is_empty(),

            _ => true,
        }
    }

    // ============================================================
    // NUMERIC OPERATIONS
    // ============================================================

    pub fn binary_numeric_op(a: Value, b: Value, op: NumericOp) -> Result<Value, RuntimeError> {
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => Self::number_op(a, b, op),

            _ => Err(RuntimeError::TypeError),
        }
    }

    pub fn number_op(a: f64, b: f64, op: NumericOp) -> Result<Value, RuntimeError> {
        match op {
            NumericOp::Add => Ok(Value::Number(a + b)),

            NumericOp::Subtract => Ok(Value::Number(a - b)),

            NumericOp::Multiply => Ok(Value::Number(a * b)),

            NumericOp::Divide => {
                if b == 0.0 {
                    return Err(RuntimeError::DivisionByZero);
                }

                Ok(Value::Number(a / b))
            }

            NumericOp::Modulo => Ok(Value::Number(a % b)),
        }
    }

    pub fn negate_values(a: Value) -> Result<Value, String> {
        match a {
            Value::Number(a) => Ok(Value::Number(-a)),

            _ => Err("Operand must be number".into()),
        }
    }

    // ============================================================
    // COMPARISON
    // ============================================================

    pub fn compare_numeric(a: Value, b: Value, op: ComparisonOp) -> Result<Value, RuntimeError> {
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
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
            (Value::Number(a), Value::Number(b)) => a == b,

            (Value::Boolean(a), Value::Boolean(b)) => a == b,

            (Value::String(a), Value::String(b)) => a == b,

            (Value::Nil, Value::Nil) => true,

            _ => false,
        }
    }
}
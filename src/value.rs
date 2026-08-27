use crate::{compiler::Function, error::RuntimeError, machine::Closure, native::NativeFn};
use std::{cell::RefCell, fmt::Display, rc::Rc};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Upvalue {
    pub location: Rc<RefCell<Value>>,
}
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

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    String(String),
    Function(Rc<Function>),
    Closure(Rc<RefCell<Closure>>),
    Nil,
    NativeFunction(NativeFn),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(value) => write!(f, "{value}"),
            Value::Boolean(value) => write!(f, "{value}"),
            Value::Nil => write!(f, "nil"),
            Value::String(s) => write!(f, "{s}"),
            Value::Function(function) => write!(f, "<fun '{}'>", function.name),
            Value::Closure(closure) => {
                let closure = closure.borrow();
                write!(f, "<closure '{}'>", closure.function.name)?;
                Ok(())
            }
            Value::NativeFunction(function) => {
                write!(f, "<nativeFn '{:?}  '>", function)?;
                Ok(())
            }
        }
    }
}
impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Boolean(value) => *value,
            _ => true,
        }
    }
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

    pub fn equals(a: Value, b: Value) -> bool {
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }
}

pub fn print_value(value: Value) {
    match value {
        Value::Number(n) => print!("{}", n),
        Value::Boolean(b) => print!("{}", b),
        Value::String(s) => print!("{}", s),
        Value::Nil => print!("nil"),
        Value::Function(function) => print!("<func: {}>", function.name),
        Value::Closure(closure) => {
            let closure = closure.borrow();
            print!("<closure '{}'>", closure.function.name)
        }
        Value::NativeFunction(function) => {
            print!("<fun '{:?} '>", function)
        }
    }
}

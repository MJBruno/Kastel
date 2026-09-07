use std::{cell::RefCell, rc::Rc};

use crate::{runtime::function::Function, runtime::upvalue::ObjUpvalue};

#[derive(Debug, Clone, PartialEq)]
pub struct Closure {
    pub function: Rc<Function>,
    pub upvalues: Vec<Rc<RefCell<ObjUpvalue>>>,
}

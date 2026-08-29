use std::{cell::RefCell, rc::Rc};

 
use crate::{runtime::function::Function, vm::machine::ObjUpvalue};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct Closure {
    pub function: Rc<Function>,
    pub upvalues: Vec<Rc<RefCell<ObjUpvalue>>>,
}

use std::{cell::RefCell, rc::Rc};

use crate::{function::Function, machine::ObjUpvalue};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Closure {
    pub function: Rc<Function>,
    pub upvalues: Vec<Rc<RefCell<ObjUpvalue>>>,
}

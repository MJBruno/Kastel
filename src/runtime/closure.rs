use std::{cell::RefCell, rc::Rc};

use crate::runtime::{function::Function, gc_handle::Gc, object::Object, upvalue::ObjUpvalue};

#[derive(Debug, Clone, PartialEq)]
pub struct Closure {
    pub function: Rc<Function>,
    pub upvalues: Vec<Rc<RefCell<ObjUpvalue>>>,
    /// Classe qui possède cette méthode.
    ///
    /// `None` pour les closures ordinaires.
    pub owner_class: Option<Gc<Object>>,
}

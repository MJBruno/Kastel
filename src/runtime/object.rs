use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::module::module::ModuleInstance;
use crate::runtime::closure::Closure;
use crate::runtime::function::Function;
use crate::runtime::gc_handle::Gc;
use crate::runtime::iterator::IteratorState;
use crate::runtime::upvalue::ObjUpvalue;
use crate::runtime::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    String(String),

    Array(Vec<Value>),

    Dict(Vec<(Value, Value)>),

    Function(Rc<Function>),

    Closure(Closure),

    Iterator(IteratorState),

    Module(Rc<ModuleInstance>),

    Class {
        name: String,
        superclass: Option<Gc<Object>>,
        interfaces: Vec<Gc<Object>>,
        methods: HashMap<String, Value>,
    },

    Interface {
        name: String,
        methods: HashMap<String, usize>,
    },

    Instance {
        class: Gc<Object>,
        fields: HashMap<String, Value>,
    },
}

impl Object {
    pub fn new_closure(
        function: Rc<Function>,
        upvalues: Vec<Rc<RefCell<ObjUpvalue>>>,
    ) -> Gc<Object> {
        let handle = Gc::new(Object::Closure(Closure {
            function,
            upvalues,
            owner_class: None,
        }));
        crate::runtime::gc::register_object(&handle);

        handle
    }

    pub(crate) fn break_cycle(&mut self) {
        match self {
            Object::String(_) => {}

            Object::Array(elements) => {
                elements.clear();
            }

            Object::Dict(fields) => {
                fields.clear();
            }

            Object::Function(_) => {}

            Object::Closure(closure) => {
                closure.upvalues.clear();
                closure.owner_class = None;
            }

            Object::Iterator(state) => {
                state.reset_for_gc();
            }

            Object::Module(_) => {}

            Object::Class {
                interfaces,
                methods,
                ..
            } => {
                interfaces.clear();
                methods.clear();
            }

            Object::Interface { methods, .. } => {
                methods.clear();
            }

            Object::Instance { fields, .. } => {
                fields.clear();
            }
        }
    }
}

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use crate::error::runtime_error::RuntimeError;
use crate::module::module::ModuleLoader;
use crate::runtime::function::Function;
use crate::runtime::gc_handle::Gc;
use crate::runtime::native::register_natives;
use crate::runtime::object::Object;
use crate::runtime::upvalue::ObjUpvalue;
use crate::runtime::value::Value;

pub mod arithmetic;
pub mod bytecode;
pub mod calls;
pub mod closures;
pub mod collections;
pub mod control_flow;
pub mod debug;
pub mod dispatch;
pub mod execution;
pub mod gc;
pub mod iterators;
pub mod modules;
pub mod stack;
pub mod variables;

pub struct CallFrame {
    pub(crate) closure: Gc<Object>,
    pub(crate) ip: usize,
    pub(crate) slot_start: usize,
}

#[allow(dead_code)]
pub struct VirtualMachine {
    pub stack: Vec<Value>,
    pub globals: HashMap<String, Value>,
    pub(crate) frames: Vec<CallFrame>,
    pub(crate) open_upvalues: Vec<Rc<RefCell<ObjUpvalue>>>,
    pub(crate) natives: HashMap<String, Value>,
    pub module_loader: ModuleLoader,
    pub module_path: Option<PathBuf>,
    pub current_line: usize,
    pub current_column: usize,
}

impl VirtualMachine {
    pub fn new(function: Rc<Function>, module_path: Option<PathBuf>) -> Self {
        let closure = Object::new_closure(function, Vec::new());

        let mut vm = Self {
            stack: vec![Value::Nil],
            globals: HashMap::new(),
            frames: vec![CallFrame {
                closure,
                ip: 0,
                slot_start: 0,
            }],
            open_upvalues: Vec::new(),
            natives: HashMap::new(),
            module_loader: ModuleLoader::new(),
            module_path,
            current_line: 0,
            current_column: 0,
        };

        register_natives(&mut vm.globals);
        vm
    }

    pub fn execute_module(
        function: Rc<Function>,
        exports: &[String],
        module_path: PathBuf,
    ) -> Result<HashMap<String, Value>, RuntimeError> {
        let mut vm = Self::new(function, Some(module_path));

        if let Err(error) = vm.run() {
            return Err(RuntimeError::WithLocation {
                line: vm.current_line,
                column: vm.current_column,
                source: Box::new(error),
            });
        }

        let mut values = HashMap::with_capacity(exports.len());

        for name in exports {
            let value = vm
                .globals
                .get(name)
                .cloned()
                .ok_or(RuntimeError::ModuleError(format!(
                    "Export '{}' was not initialized",
                    name
                )))?;

            values.insert(name.clone(), value);
        }

        Ok(values)
    }

   
pub(crate) fn capture_upvalue(
    &mut self,
    slot: usize,
) -> Result<Rc<RefCell<ObjUpvalue>>, RuntimeError> {
    let slot_start = self.current_frame()?.slot_start;

    let absolute_slot = slot_start
        .checked_add(1)
        .and_then(|value| value.checked_add(slot))
        .ok_or(RuntimeError::InvalidFunction)?;

    if absolute_slot >= self.stack.len() {
        return Err(RuntimeError::InvalidFunction);
    }

    for upvalue in &self.open_upvalues {
        if upvalue.borrow().slot == absolute_slot {
            return Ok(Rc::clone(upvalue));
        }
    }

    let upvalue = Rc::new(
        RefCell::new(ObjUpvalue::new(absolute_slot))
    );

    crate::runtime::gc::register_upvalue(&upvalue);
    self.open_upvalues.push(Rc::clone(&upvalue));

    Ok(upvalue)
}


    pub(crate) fn close_upvalues(&mut self, last: usize) {
        let mut remaining = Vec::with_capacity(self.open_upvalues.len());

        for upvalue in self.open_upvalues.drain(..) {
            let slot = upvalue.borrow().slot;

            if slot >= last {
                let value = self.stack.get(slot).cloned().unwrap_or(Value::Nil);

                upvalue.borrow_mut().closed = Some(value);
            } else {
                remaining.push(upvalue);
            }
        }

        self.open_upvalues = remaining;
    }
}

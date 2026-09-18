use super::VirtualMachine;
use crate::runtime::gc;

impl VirtualMachine {
    pub fn collect_garbage(&mut self) -> usize {
        let globals = self.globals.borrow();
        let modules = self.module_loader.loaded_modules();

        gc::collect(gc::GcRoots {
            stack: &self.stack,
            globals: &globals,
            modules: &modules,
            frames: &self.frames,
            open_upvalues: &self.open_upvalues,
            pending_exception: &self.pending_exception,
        })
    }
}

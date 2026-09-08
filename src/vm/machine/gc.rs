use super::VirtualMachine;
use crate::runtime::gc;

impl VirtualMachine {
    pub fn collect_garbage(&mut self) -> usize {
        gc::collect(gc::GcRoots {
            stack: &self.stack,
            globals: &self.globals,
            frames: &self.frames,
            open_upvalues: &self.open_upvalues,
        })
    }
}

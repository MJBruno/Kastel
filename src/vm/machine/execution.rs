use super::VirtualMachine;
use crate::error::runtime_error::RuntimeError;
use crate::runtime::gc;

impl VirtualMachine {
    pub fn run(&mut self) -> Result<(), RuntimeError> {
        loop {
            if cfg!(feature = "debug_trace") {
                self.debug_machine()?;
            }

            if gc::should_collect() {
                self.collect_garbage();
            }

            let (line, column) = self.current_position()?;
            self.current_line = line;
            self.current_column = column;

            let instruction = self.read_byte()?;

            if self.dispatch(instruction)? {
                return Ok(());
            }
        }
    }
}

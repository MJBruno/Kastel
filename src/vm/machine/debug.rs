use super::VirtualMachine;
use super::bytecode::frame_closure;
use crate::error::runtime_error::RuntimeError;

impl VirtualMachine {
    fn print_stack(&self) {
        print!("          ");
        for value in &self.stack {
            print!(" [ {} ]", value);
        }
        println!();
    }

    pub(crate) fn debug_machine(&mut self) -> Result<(), RuntimeError> {
        self.print_stack();
        let (ip, chunk) = {
            let frame = self.current_frame()?;
            let closure = frame_closure(&frame.closure);
            (frame.ip, closure.function.chunk.clone())
        };
        chunk.disassemble_instruction(ip);
        Ok(())
    }
}

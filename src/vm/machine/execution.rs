use super::VirtualMachine;

use crate::error::runtime_error::RuntimeError;
use crate::runtime::gc;
use crate::runtime::value::Value;

impl VirtualMachine {
    // ============================================================
    // RUN
    // ============================================================

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

            match self.dispatch(instruction) {
                Ok(true) => {
                    return Ok(());
                }

                Ok(false) => {}

                Err(error) => {
                    if !self.propagate_runtime_error(error.clone())? {
                        return Err(error);
                    }
                }
            }
        }
    }

    // ============================================================
    // RUNTIME ERROR PROPAGATION
    // ============================================================

    pub(crate) fn propagate_runtime_error(
        &mut self,
        error: RuntimeError,
    ) -> Result<bool, RuntimeError> {
        match error {
            RuntimeError::Thrown(value) => self.propagate_thrown(value),

            error => Err(error),
        }
    }

    // ============================================================
    // THROW PROPAGATION
    // ============================================================

    pub(crate) fn propagate_thrown(&mut self, value: Value) -> Result<bool, RuntimeError> {
        loop {
            // ----------------------------------------------------
            // Plus aucun frame
            // ----------------------------------------------------

            if self.frames.is_empty() {
                return Ok(false);
            }

            let current_frame_index = self.frames.len() - 1;

            // ----------------------------------------------------
            // Trouver le handler actif le plus proche
            // ----------------------------------------------------

            let handler_index = self
                .exception_handlers
                .iter()
                .rposition(|handler| handler.frame_index <= current_frame_index);

            let Some(handler_index) = handler_index else {
                // Aucun handler dans cette frame :
                // remonter vers l'appelant.
                self.close_current_frame_for_exception()?;
                continue;
            };

            // ----------------------------------------------------
            // Consommer le handler
            // ----------------------------------------------------

            let handler = self.exception_handlers.remove(handler_index);

            // ----------------------------------------------------
            // Remonter les frames jusqu'à celle du handler
            // ----------------------------------------------------

            while self.frames.len() - 1 > handler.frame_index {
                self.close_current_frame_for_exception()?;
            }

            if self.frames.is_empty() {
                return Ok(false);
            }

            // ----------------------------------------------------
            // Restaurer la stack au point d'entrée du try
            // ----------------------------------------------------

            self.restore_exception_stack(handler.stack_height)?;

            // ----------------------------------------------------
            // CATCH
            // ----------------------------------------------------

            if let Some(catch_ip) = handler.catch_ip {
                // catch(e) récupérera cette valeur depuis la stack.
                self.push(value);

                self.current_frame_mut()?.ip = catch_ip;

                return Ok(true);
            }

            // ----------------------------------------------------
            // FINALLY
            // ----------------------------------------------------

            if let Some(finally_ip) = handler.finally_ip {
                self.pending_exception = Some(super::PendingException {
                    value,
                    rethrow: true,
                });

                self.current_frame_mut()?.ip = finally_ip;

                return Ok(true);
            }

            // ----------------------------------------------------
            // Handler sans catch/finally
            // Continuer la propagation.
            // ----------------------------------------------------
        }
    }
}

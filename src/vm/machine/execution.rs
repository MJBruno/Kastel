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

    pub(crate) fn propagate_runtime_error(
        &mut self,
        error: RuntimeError,
    ) -> Result<bool, RuntimeError> {
        match error {
            RuntimeError::Thrown(value) => self.propagate_thrown(value),

            error => {
                let value = Value::new_string(error.to_string());

                self.propagate_thrown(value)
            }
        }
    }

   

    
    // // ============================================================
    // // RUNTIME ERROR -> KASTEL VALUE
    // // ============================================================

    // fn runtime_error_to_value(error: &RuntimeError) -> Value {
    //     Value::new_string(error.to_string())
    // }

    // // ============================================================
    // // UNCAUGHT ERROR
    // // ============================================================

    // fn pending_runtime_error(&self) -> RuntimeError {
    //     /*
    //      * Cette méthode sert uniquement à conserver une erreur
    //      * RuntimeError normale lorsque aucune exception Kastel
    //      * n'a pu être capturée.
    //      *
    //      * Le système actuel ne conserve pas la RuntimeError originale
    //      * séparément du Value généré par runtime_error_to_value().
    //      *
    //      * Pour une exception explicite `throw`, propagate_thrown()
    //      * retourne directement la vraie Value.
    //      */
    //     RuntimeError::NativeError
    // }

    // ============================================================
    // THROW PROPAGATION
    // ============================================================

    pub(crate) fn propagate_thrown(&mut self, value: Value) -> Result<bool, RuntimeError> {
        loop {
            /*
             * Aucun frame :
             *
             * exception non capturée.
             */
            if self.frames.is_empty() {
                return Ok(false);
            }

            let current_frame_index = self.frames.len() - 1;

            /*
             * Chercher le handler le plus proche.
             */
            let handler_index = self
                .exception_handlers
                .iter()
                .rposition(|handler| handler.frame_index <= current_frame_index);

            let Some(handler_index) = handler_index else {
                /*
                 * Aucun handler dans les frames actuels.
                 *
                 * Remonter d'un frame.
                 */
                self.close_current_frame_for_exception()?;

                continue;
            };

            /*
             * Retirer le handler.
             *
             * Il ne doit plus être actif pendant catch/finally.
             */
            let handler = self.exception_handlers.remove(handler_index);

            /*
             * --------------------------------------------------------
             * REMONTÉE DES FRAMES
             * --------------------------------------------------------
             */
            while self.frames.len() - 1 > handler.frame_index {
                self.close_current_frame_for_exception()?;
            }

            if self.frames.is_empty() {
                return Ok(false);
            }

            /*
             * --------------------------------------------------------
             * RESTAURATION STACK
             * --------------------------------------------------------
             */
            self.restore_exception_stack(handler.stack_height)?;

            /*
             * --------------------------------------------------------
             * CATCH
             * --------------------------------------------------------
             */
            if let Some(catch_ip) = handler.catch_ip {
                /*
                 * La valeur de l'exception devient la
                 * valeur disponible pour catch(e).
                 */
                self.push(value);

                self.current_frame_mut()?.ip = catch_ip;

                return Ok(true);
            }

            /*
             * --------------------------------------------------------
             * FINALLY
             * --------------------------------------------------------
             */
            if let Some(finally_ip) = handler.finally_ip {
                self.pending_exception = Some(super::PendingException {
                    value,
                    rethrow: true,
                });

                self.current_frame_mut()?.ip = finally_ip;

                return Ok(true);
            }

            /*
             * Handler vide :
             * continuer la propagation.
             */
        }
    }
}

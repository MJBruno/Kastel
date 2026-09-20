use super::VirtualMachine;
use crate::runtime::gc;
use crate::runtime::value::Value;
use crate::error::runtime_error::RuntimeError;

impl VirtualMachine {
    pub fn collect_garbage(&mut self) -> usize {
        let globals = self.globals.borrow();
        let modules = self.module_loader.loaded_modules();

        gc::collect(gc::GcRoots {
            temp: &self.temp_roots,
            stack: &self.stack,
            globals: &globals,
            modules: &modules,
            frames: &self.frames,
            open_upvalues: &self.open_upvalues,
            pending_exception: &self.pending_exception,
        })
    }

    /// Épingle les racines de CETTE VM tant que la garde renvoyée existe.
    ///
    /// À prendre avant de lancer une VM imbriquée (`import` : le module
    /// s'exécute dans sa propre VM) : ses collectes verront ainsi aussi la
    /// pile, les globales et les frames de la VM appelante, qui restent
    /// figées pendant ce temps.
    pub(crate) fn pin_roots(&self) -> gc::PinnedRoots {
        let mut values = Vec::with_capacity(self.stack.len() + self.temp_roots.len());

        values.extend(self.stack.iter().cloned());
        values.extend(self.temp_roots.iter().cloned());
        values.extend(self.globals.borrow().values().cloned());

        for frame in &self.frames {
            values.push(Value::Object(frame.closure.clone()));
        }

        if let Some(exception) = &self.pending_exception {
            values.push(exception.value.clone());
        }

        gc::pin_roots(gc::ExternalRoots {
            values,
            upvalues: self.open_upvalues.clone(),
        })
    }

    // ============================================================
    //              RACINES TEMPORAIRES (code natif)
    // ============================================================
    //
    // Le GC ne collecte qu'entre deux instructions (et pendant `invoke_sync`).
    // Toute valeur tenue UNIQUEMENT par une variable Rust pendant un rappel
    // Kastel doit donc être enracinée, sinon elle serait vidée.

    /// Enracine `value` jusqu'à la fin de l'appel natif englobant
    /// (voir `with_temp_roots`).
    pub(crate) fn protect(&mut self, value: &Value) {
        self.temp_roots.push(value.clone());
    }

    /// Exécute `f` en gardant `values` enracinées ; toutes les racines
    /// ajoutées pendant `f` (par `protect`) sont retirées au retour, y
    /// compris en cas d'erreur.
    pub(crate) fn with_temp_roots<R>(
        &mut self,
        values: &[Value],
        f: impl FnOnce(&mut Self) -> Result<R, RuntimeError>,
    ) -> Result<R, RuntimeError> {
        let mark = self.temp_roots.len();

        self.temp_roots.extend(values.iter().cloned());

        let result = f(self);

        self.temp_roots.truncate(mark);

        result
    }
}

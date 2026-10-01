use super::VirtualMachine;
use crate::error::runtime_error::RuntimeError;
use crate::runtime::gc;
use crate::runtime::value::Value;

impl VirtualMachine {
    pub fn collect_garbage(&mut self) -> usize {
        let globals = self.globals.borrow();
        let modules = self.module_loader.loaded_modules();
        let mut scheduler_values = Vec::new();
        let mut scheduler_upvalues = Vec::new();

        if let Some(scheduler) = self.scheduler.upgrade() {
            scheduler
                .borrow()
                .append_gc_roots(&mut scheduler_values, &mut scheduler_upvalues);
        }

        gc::collect(gc::GcRoots {
            temp: &self.temp_roots,
            stack: &self.stack,
            globals: &globals,
            modules: &modules,
            frames: &self.frames,
            open_upvalues: &self.open_upvalues,
            extra_values: &scheduler_values,
            extra_upvalues: &scheduler_upvalues,
        })
    }

    /// Épingle les racines de CETTE VM tant que la garde renvoyée existe.
    ///
    /// À prendre avant de lancer une VM imbriquée (`import` : le module
    /// s'exécute dans sa propre VM) : ses collectes verront ainsi aussi la
    /// pile, les globales et les frames de la VM appelante, qui restent
    /// figées pendant ce temps.
    pub(crate) fn pin_roots(&self) -> gc::PinnedRoots {
        self.pin_roots_with(&[])
    }

    /// Comme `pin_roots`, en épinglant en plus `extra` : valeurs tenues
    /// uniquement par une variable Rust (récepteur dépilé, par exemple)
    /// pendant qu'une AUTRE VM (tâche) s'exécute et peut collecter.
    pub(crate) fn pin_roots_with(&self, extra: &[Value]) -> gc::PinnedRoots {
        let mut values = Vec::with_capacity(self.stack.len() + self.temp_roots.len() + extra.len());

        values.extend(extra.iter().cloned());

        values.extend(self.stack.iter().cloned());
        values.extend(self.temp_roots.iter().cloned());
        values.extend(self.globals.borrow().values().cloned());

        for frame in &self.frames {
            values.push(Value::Object(frame.closure.clone()));
        }

        // Valeur lancée par une autre tâche (join/recv...) et pas encore
        // livrée : elle n'est plus qu'ici jusqu'à la prochaine exécution.
        if let Some(error) = &self.waiting_error {
            super::scheduler::Scheduler::root_runtime_error(error, &mut values);
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

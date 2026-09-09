use crate::bytecode::chunk::OpCode;

use super::compiler::Compiler;

impl Compiler {
    // ============================================================
    // SCOPE
    // ============================================================

    pub(crate) fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    pub(crate) fn end_scope(&mut self) {
        self.scope_depth -= 1;

        let count = self
            .context
            .borrow_mut()
            .locals
            .pop_scope(self.scope_depth);

        for _ in 0..count {
            self.emit_opcode(OpCode::Pop);
        }
    }

    /// Émet les Pop nécessaires pour quitter les scopes
    /// jusqu'à `target_depth`.
    ///
    /// Cette fonction ne modifie PAS la table des locals.
    pub(crate) fn emit_scope_cleanup(
        &mut self,
        target_depth: usize,
    ) {
        let count = self
            .context
            .borrow()
            .locals
            .cleanup_count(target_depth);

        for _ in 0..count {
            self.emit_opcode(OpCode::Pop);
        }
    }

    /// Ferme uniquement le scope côté compilateur.
    ///
    /// Les Pop ont déjà été émis par `emit_scope_cleanup()`.
    pub(crate) fn discard_scope(&mut self) {
        self.scope_depth -= 1;

        self.context
            .borrow_mut()
            .locals
            .pop_scope(self.scope_depth);
    }
}


use super::VirtualMachine;

use crate::{
    error::runtime_error::RuntimeError,
    runtime::{object::Object, value::Value},
};

impl VirtualMachine {
    pub(crate) fn op_push_exception_handler(&mut self) -> Result<(), RuntimeError> {
        let catch_raw = self.read_short()?;
        let finally_raw = self.read_short()?;
        let catch_type_raw = self.read_short()?;

        let catch_ip = if catch_raw == u16::MAX {
            None
        } else {
            Some(catch_raw as usize)
        };

        let finally_ip = if finally_raw == u16::MAX {
            None
        } else {
            Some(finally_raw as usize)
        };

        let catch_type = if catch_type_raw == u16::MAX {
            None
        } else {
            let value = self.read_constant(catch_type_raw)?;
            Some(value.as_string_value().ok_or(RuntimeError::TypeError)?)
        };

        self.register_exception_handler(catch_ip, finally_ip, catch_type)
    }

    pub(crate) fn op_pop_exception_handler(&mut self) -> Result<(), RuntimeError> {
        self.unregister_exception_handler().map(|_| ())
    }

    pub(crate) fn op_throw(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop()?;

        Err(RuntimeError::Thrown(value))
    }

    /// Fin d'un bloc `finally`.
    ///
    /// L'état du `finally` vit sur la PILE, sous forme de deux locales cachées
    /// `[valeur, drapeau]` empilées à l'entrée du bloc :
    ///
    /// - entrée normale (fin du `try` ou du `catch`) : `[None, false]` ;
    /// - entrée par exception : `[valeur lancée, true]`.
    ///
    /// Contrairement à l'ancien emplacement unique `pending_exception`, un
    /// `finally` imbriqué (ou appelé depuis un `finally`) ne peut plus voler ni
    /// écraser l'exception en attente d'un `finally` englobant, et la valeur
    /// est automatiquement une racine du GC.
    pub(crate) fn op_finally_end(&mut self) -> Result<(), RuntimeError> {
        let flag = self.pop()?;
        let value = self.pop()?;

        if !matches!(flag, Value::Boolean(true)) {
            return Ok(());
        }

        if self.cancelling && Self::is_cancellation_value(&value) {
            return Err(RuntimeError::TaskCancelled);
        }

        Err(RuntimeError::Thrown(value))
    }

    /// Valeur empilée à l'entrée d'un `finally` lors d'une annulation.
    pub(crate) fn cancellation_value() -> Value {
        Value::new_error(
            RuntimeError::TaskCancelled.kind_name(),
            RuntimeError::TaskCancelled.to_string(),
        )
    }

    fn is_cancellation_value(value: &Value) -> bool {
        match value {
            Value::Object(handle) => match &*handle.borrow() {
                Object::Error { kind, .. } => {
                    kind.as_str() == RuntimeError::TaskCancelled.kind_name()
                }
                _ => false,
            },
            _ => false,
        }
    }

    /// Vrai si un handler de cette VM a un `finally` à exécuter (la tâche doit
    /// alors être « déroulée » plutôt que détruite brutalement à l'annulation).
    pub(crate) fn has_finally_handlers(&self) -> bool {
        self.exception_handlers
            .iter()
            .any(|handler| handler.finally_ip.is_some())
    }

    /// Prépare une annulation COOPÉRATIVE : la prochaine exécution lève
    /// `TaskCancelled`, qui exécute uniquement les `finally` (jamais les
    /// `catch`) avant de terminer la tâche.
    pub(crate) fn begin_cancellation(&mut self) {
        self.cancelling = true;
        self.waiting_requested = false;
        self.yield_requested = false;
        self.waiting_channel = None;
        self.waiting_channel_send = None;
        self.waiting_select_channels = None;
        self.waiting_select_send_values = None;
        self.waiting_timer = None;
        self.waiting_mutex = None;
        self.waiting_semaphore = None;
        self.waiting_wait_group = None;
        self.waiting_barrier = None;
        self.waiting_rwlock = None;
        self.waiting_event = None;
        self.waiting_condvar = None;
        self.waiting_task = None;
        self.waiting_error = Some(RuntimeError::TaskCancelled);
    }
}

use std::collections::VecDeque;

/// Événement coopératif à état permanent.
///
/// `set()` passe l'événement à l'état signalé et réveille tous les
/// waiters. Une fois signalé, `wait()` ne bloque plus tant que `reset()`
/// n'a pas été appelé.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventState {
    pub(crate) signaled: bool,
    pub(crate) waiters: VecDeque<usize>,
}

impl Default for EventState {
    fn default() -> Self {
        Self::new()
    }
}

impl EventState {
    pub fn new() -> Self {
        Self {
            signaled: false,
            waiters: VecDeque::new(),
        }
    }

    pub fn is_set(&self) -> bool {
        self.signaled
    }
}

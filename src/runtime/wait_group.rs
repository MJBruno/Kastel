use std::collections::VecDeque;

/// Primitive de coordination coopérative : un compteur de tâches.
/// `wait()` bloque tant que le compteur est strictement positif.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaitGroupState {
    pub(crate) count: usize,
    pub(crate) waiters: VecDeque<usize>,
}

impl Default for WaitGroupState {
    fn default() -> Self {
        Self::new()
    }
}

impl WaitGroupState {
    pub fn new() -> Self {
        Self {
            count: 0,
            waiters: VecDeque::new(),
        }
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn is_done(&self) -> bool {
        self.count == 0
    }
}

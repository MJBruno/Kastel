use std::collections::VecDeque;

/// Verrou coopératif non réentrant.
///
/// Le propriétaire est l'identifiant de la tâche Kastel qui a acquis le
/// verrou. Les tâches en attente sont réveillées dans l'ordre FIFO.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutexState {
    pub(crate) owner: Option<usize>,
    pub(crate) waiters: VecDeque<usize>,
}

impl Default for MutexState {
    fn default() -> Self {
        Self::new()
    }
}

impl MutexState {
    pub fn new() -> Self {
        Self {
            owner: None,
            waiters: VecDeque::new(),
        }
    }

    pub fn is_locked(&self) -> bool {
        self.owner.is_some()
    }
}

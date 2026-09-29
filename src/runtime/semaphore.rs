use std::collections::VecDeque;

/// Sémaphore coopératif à compteur. Chaque `acquire()` consomme un permis ;
/// les tâches bloquées sont réveillées dans l'ordre FIFO par `release()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemaphoreState {
    pub(crate) permits: usize,
    pub(crate) capacity: usize,
    pub(crate) waiters: VecDeque<usize>,
}

impl SemaphoreState {
    pub fn new(capacity: usize) -> Self {
        Self {
            permits: capacity,
            capacity,
            waiters: VecDeque::new(),
        }
    }

    pub fn available(&self) -> usize {
        self.permits
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

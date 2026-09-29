use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::runtime::mutex::MutexState;

/// Condition variable coopérative associée à un mutex.
///
/// Les tâches attendent en FIFO. `wait()` libère atomiquement le mutex dans
/// le scheduler, puis la notification fait repasser la tâche par la file du
/// mutex avant qu'elle ne reprenne l'exécution après `wait()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CondvarState {
    pub(crate) mutex: Rc<RefCell<MutexState>>,
    pub(crate) waiters: VecDeque<usize>,
}

impl CondvarState {
    pub fn new(mutex: Rc<RefCell<MutexState>>) -> Self {
        Self {
            mutex,
            waiters: VecDeque::new(),
        }
    }

    pub fn waiter_count(&self) -> usize {
        self.waiters.len()
    }
}

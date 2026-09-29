use std::collections::VecDeque;

use crate::runtime::value::Value;

/// Canal coopératif. `None` représente un canal non borné ; `Some(n)` un
/// canal borné à `n` éléments. Le canal reste indépendant des tâches : les
/// files d'attente de producteurs/consommateurs sont gérées par le scheduler.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelState {
    pub(crate) queue: VecDeque<Value>,
    pub(crate) closed: bool,
    pub(crate) capacity: Option<usize>,
}

impl Default for ChannelState {
    fn default() -> Self {
        Self::new()
    }
}

impl ChannelState {
    /// Construit un canal non borné.
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            closed: false,
            capacity: None,
        }
    }

    /// Construit un canal borné à `capacity` éléments.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            queue: VecDeque::new(),
            closed: false,
            capacity: Some(capacity),
        }
    }

    pub fn send(&mut self, value: Value) {
        debug_assert!(
            self.capacity
                .map_or(true, |capacity| self.queue.len() < capacity),
            "send() appelé sur un canal borné déjà plein"
        );
        self.queue.push_back(value);
    }

    pub fn is_full(&self) -> bool {
        self.capacity
            .is_some_and(|capacity| self.queue.len() >= capacity)
    }

    pub fn capacity(&self) -> Option<usize> {
        self.capacity
    }

    /// Ferme le canal. Les valeurs déjà en file restent recevables.
    pub fn close(&mut self) -> bool {
        if self.closed {
            return false;
        }

        self.closed = true;
        true
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn try_recv(&mut self) -> Option<Value> {
        self.queue.pop_front()
    }

    pub fn size(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

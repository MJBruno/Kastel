use std::collections::VecDeque;

use crate::runtime::value::Value;

/// Canal coopératif non borné.
///
/// `try_recv()` est non bloquant. La primitive bloquante `recv()` est gérée
/// par la VM et le scheduler : le canal lui-même ne connaît pas les tâches.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelState {
    pub(crate) queue: VecDeque<Value>,
    pub(crate) closed: bool,
}

impl Default for ChannelState {
    fn default() -> Self {
        Self::new()
    }
}

impl ChannelState {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            closed: false,
        }
    }

    pub fn send(&mut self, value: Value) {
        self.queue.push_back(value);
    }

    /// Ferme le canal. Les valeurs déjà en file restent recevables.
    /// Retourne `true` si l'état est passé d'ouvert à fermé.
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

use std::collections::VecDeque;

use crate::runtime::value::Value;

/// Canal coopératif non borné.
///
/// `try_recv()` est non bloquant. La primitive bloquante `recv()` est gérée
/// par la VM et le scheduler : le canal lui-même ne connaît pas les tâches.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelState {
    pub(crate) queue: VecDeque<Value>,
}

impl ChannelState {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn send(&mut self, value: Value) {
        self.queue.push_back(value);
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

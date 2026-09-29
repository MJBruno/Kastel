use std::collections::VecDeque;

/// Barrière coopérative réutilisable à nombre fixe de participants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarrierState {
    pub(crate) parties: usize,
    pub(crate) arrived: usize,
    pub(crate) generation: usize,
    pub(crate) broken: bool,
    pub(crate) waiters: VecDeque<usize>,
}

impl BarrierState {
    pub fn new(parties: usize) -> Self {
        Self {
            parties,
            arrived: 0,
            generation: 0,
            broken: false,
            waiters: VecDeque::new(),
        }
    }

    pub fn parties(&self) -> usize {
        self.parties
    }
    pub fn arrived(&self) -> usize {
        self.arrived
    }
    pub fn generation(&self) -> usize {
        self.generation
    }
    pub fn is_broken(&self) -> bool {
        self.broken
    }
}

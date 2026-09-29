use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RwLockMode {
    Read,
    Write,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RwLockState {
    pub(crate) writer: Option<usize>,
    pub(crate) readers: Vec<usize>,
    pub(crate) waiters: VecDeque<(usize, RwLockMode)>,
}

impl Default for RwLockState {
    fn default() -> Self {
        Self::new()
    }
}

impl RwLockState {
    pub fn new() -> Self {
        Self {
            writer: None,
            readers: Vec::new(),
            waiters: VecDeque::new(),
        }
    }

    pub fn reader_count(&self) -> usize {
        self.readers.len()
    }

    pub fn is_read_locked(&self) -> bool {
        !self.readers.is_empty()
    }

    pub fn is_write_locked(&self) -> bool {
        self.writer.is_some()
    }
}

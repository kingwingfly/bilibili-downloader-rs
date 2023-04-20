//! This is a helper for Task
//! to record the process of downloading task

use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub struct Process {
    pub total: AtomicUsize,
    pub finished: AtomicUsize,
}

impl Process {
    pub fn new() -> Process {
        Process {
            total: AtomicUsize::new(0),
            finished: AtomicUsize::new(0),
        }
    }

    pub fn add_total(&self, val: usize) {
        self.total.fetch_add(val, Ordering::SeqCst);
    }

    pub fn add_finished(&self, val: usize) {
        self.finished.fetch_add(val, Ordering::SeqCst);
    }

    pub fn total(&self) -> usize {
        self.total.load(Ordering::Relaxed)
    }

    pub fn finished(&self) -> usize {
        self.finished.load(Ordering::SeqCst)
    }
}

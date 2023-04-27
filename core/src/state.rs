//! This is state recorder;
//! Implement by Finite State Machine (FSM)

use std::sync::atomic::{AtomicUsize, Ordering};

// working pausing cancelled finished
const STATENUM: usize = 4;
// siwtch cancel finish
const TRIGGERNUM: usize = 3;

#[derive(Debug)]
pub struct FSM {
    matrix: [[usize; STATENUM]; TRIGGERNUM],
    c: AtomicUsize,
}

impl FSM {
    pub fn new() -> Self {
        let matrix = [
            // working pausing cancelled finished
            /*siwtch*/ [1, 0, 2, 3],
            /*cancel*/ [2, 2, 2, 3],
            /*finish*/ [3, 3, 2, 3],
        ];
        Self {
            matrix,
            c: AtomicUsize::new(0),
        }
    }

    pub fn now_state_code(&self) -> usize {
        self.c.load(Ordering::Relaxed)
    }

    #[allow(unused)]
    fn now(&self) -> State {
        match self.c.load(Ordering::Relaxed) {
            0 => State::Working,
            1 => State::Pausing,
            2 => State::Cancelled,
            3 => State::Finished,
            _ => unreachable!(),
        }
    }

    pub fn switch(&self) {
        self.change_state(0);
    }

    pub fn cancel(&self) {
        self.change_state(1);
    }

    pub fn finish(&self) {
        self.change_state(2);
    }

    fn change_state(&self, trigger: usize) {
        let c = self.c.load(Ordering::Relaxed);
        let new = self.matrix[trigger][c];
        self.c
            .fetch_update(Ordering::SeqCst, Ordering::Relaxed, |_| Some(new))
            .unwrap();
    }
}

#[derive(Debug)]
pub enum State {
    Working,
    Pausing,
    Cancelled,
    Finished,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let fsm = FSM::new();
        dbg!("init", fsm.now());
        fsm.switch();
        dbg!("switch", fsm.now());
        fsm.switch();
        dbg!("switch", fsm.now());
        fsm.cancel();
        dbg!("cancel", fsm.now());
        fsm.switch();
        dbg!("switch after cancel", fsm.now());
    }
}

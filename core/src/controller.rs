//! Control the switch and cancel
//! through a watch channel

use tokio::sync::watch;

#[derive(Debug)]
pub struct Controller {
    tx: watch::Sender<bool>,
}

impl Controller {
    pub fn new(tx: watch::Sender<bool>) -> Self {
        Self { tx }
    }

    pub fn switch(&self) {
        self.tx.send_modify(|_| {});
    }

    pub fn cancel(&self) {
        self.tx.send_modify(|state| {
            // In task, if true, then cancel
            *state = true;
        });
    }
}

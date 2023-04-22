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
        self.tx.send(true).unwrap();
    }
}

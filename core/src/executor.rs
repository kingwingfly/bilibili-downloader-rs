use std::{collections::HashMap, sync::Arc};

use tokio::sync::mpsc;

use crate::{helper, message::Message, task::Task};

#[derive(Debug)]
pub struct Executor {
    tx: mpsc::Sender<Message>,
    rt: tokio::runtime::Runtime,
}

impl Executor {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::channel(8);
        std::thread::spawn(move || {
            let rt = helper::create_rt();
            rt.block_on(async move {
                let mut tasks = HashMap::new();
                while let Some(msg) = rx.recv().await {
                    match msg {
                        // spawn a download
                        Message::Job(task) => {
                            let task = Arc::new(task);
                            tasks.insert(task.id, task.clone());
                            let task_c = task.clone();
                            tokio::spawn(async move { task_c.execute().await });
                        }
                        // query the process
                        Message::Process((tx, id)) => {
                            let process = match tasks.get(&id) {
                                Some(task) => task.process(),
                                None => format!("Unknown id {}", id),
                            };
                            tx.send(process).unwrap();
                        }
                        // cancel a download
                        Message::Cancel(id) => {
                            match tasks.remove(&id) {
                                Some(task) => {
                                    task.cancel();
                                }
                                None => println!("Unknown id {}", id),
                            };
                        }
                        Message::Switch(id) => {
                            match tasks.get(&id) {
                                Some(task) => task.switch(),
                                None => println!("Unknown id {}", id),
                            };
                        }
                        Message::SwitchAll => {
                            for task in tasks.values() {
                                task.switch();
                            }
                        }
                        Message::Terminate => {
                            for task in tasks.values() {
                                task.cancel();
                            }
                        }
                    }
                }
            });
            println!("Terminated");
        });
        Self {
            tx,
            rt: helper::create_rt(),
        }
    }

    pub fn spawn_task(&self, task: Task) {
        self.rt.block_on(self.tx.send(Message::Job(task))).unwrap();
    }

    pub fn process(&self, id: usize) -> String {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.rt
            .block_on(self.tx.send(Message::Process((tx, id))))
            .unwrap();
        self.rt.block_on(rx).unwrap()
    }

    pub fn switch(&self, id: usize) {
        self.rt.block_on(self.tx.send(Message::Switch(id))).unwrap();
    }

    pub fn switch_all(&self) {
        self.rt.block_on(self.tx.send(Message::SwitchAll)).unwrap();
    }

    pub fn cancel(&self, id: usize) {
        self.rt.block_on(self.tx.send(Message::Cancel(id))).unwrap();
    }

    pub fn terminate(&self) {
        self.rt.block_on(self.tx.send(Message::Terminate)).unwrap();
    }
}

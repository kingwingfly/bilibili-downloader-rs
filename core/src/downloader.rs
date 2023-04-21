//! Downloader
//! Create a thread and spawn tasks through `run` on it, then run them asynchronously

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::helper;
use crate::message::Message;
use crate::task::Task;

#[derive(Debug)]
pub struct Downloader {
    id_next: AtomicUsize,
    rt: tokio::runtime::Runtime,
    tx: mpsc::Sender<Message>,
}

impl Downloader {
    /// Create a Downloader
    /// # Examples
    /// ```rust
    /// use core_api::downloader::Downloader;
    /// let dl = Downloader::new();
    /// ```
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::channel(8);
        std::thread::spawn(move || {
            let rt = helper::create_rt();
            rt.block_on(async move {
                let mut tasks = HashMap::new();
                let mut jhs = HashMap::new();
                while let Some(msg) = rx.recv().await {
                    match msg {
                        // spawn a download
                        Message::Job(task) => {
                            tasks.insert(task.id, task.clone());
                            let task_c = task.clone();
                            let jh = tokio::spawn(async move { task_c.execute().await });
                            jhs.insert(task.id, jh);
                        }
                        // query the process
                        Message::Process((tx, id)) => {
                            let state = tasks.get(&id).unwrap().state();
                            tx.send(state).unwrap();
                        }
                        // cancel a download
                        Message::Cancel(id) => {
                            jhs.remove(&id).unwrap().abort();
                            tasks.remove(&id).unwrap().rm_cache();
                        }
                        Message::Terminate => {
                            for task in tasks.values() {
                                task.rm_cache();
                            }
                            break;
                        }
                    }
                }
            });
            println!("Terminated");
        });
        let rt = helper::create_rt();
        Self {
            id_next: AtomicUsize::new(0),
            tx,
            rt,
        }
    }

    /// Run a downloading task
    /// # Examples
    /// ```rust
    /// use core_api::downloader::Downloader;
    /// let dl = Downloader::new();
    /// let target = "https://www.bilibili.com/video/BV1Ao4y1b7fj/?".to_owned();
    /// let save_dir = "../tests/video_downloads".to_owned();
    /// /* A cache dir will be made right beside the `save_dir`,
    /// and video will be saved at `save_dir`.
    /// The cache dir will be removed after finished */
    /// let id = dl.run(target, save_dir);
    /// ```
    pub fn run(&self, target: String, save_dir: String) -> usize {
        let id = self.id_next.fetch_add(1, Ordering::SeqCst);
        let task = Arc::new(Task::new(id, target, save_dir));
        self.rt.block_on(self.tx.send(Message::Job(task))).unwrap();
        id
    }

    /// Helper function to display progress
    /// todo
    pub fn state(&self, id: usize) -> String {
        let (tx, rx) = tokio::sync::oneshot::channel::<String>();
        self.rt.block_on(async {
            self.tx.send(Message::Process((tx, id))).await.unwrap();
            rx.await.unwrap()
        })
    }

    /// Terminate the Downloader
    /// # Attention
    /// This will interrupt the downloading and leave the cached files undeleted.
    pub fn cancel(&self, id: usize) {
        self.rt.block_on(self.tx.send(Message::Cancel(id))).unwrap();
    }

    pub fn terminate(&self) {
        let tx_clone = self.tx.clone();
        self.rt.block_on(tx_clone.send(Message::Terminate)).unwrap();
    }
}

impl Drop for Downloader {
    fn drop(&mut self) {
        if self.tx.is_closed() {
            return;
        }
        self.terminate();
        println!("Terminating")
    }
}

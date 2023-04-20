//! Downloader
//! Create a thread and spawn tasks through `run` on it, then run them asynchronously

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::helper;
use crate::task::{Message, Task};

type TasksMap = Mutex<RefCell<HashMap<usize, Arc<Task>>>>;

#[derive(Debug)]
pub struct Downloader {
    id_next: AtomicUsize,
    tx: mpsc::Sender<Message>,
    rt: tokio::runtime::Runtime,
    tasks: TasksMap,
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
                while let Some(Message::Job(task)) = rx.recv().await {
                    // dbg!(task.as_ref());
                    tokio::spawn(async move {
                        match task.execute().await {
                            Ok(_) => println!("All Finished"),
                            Err(e) => {
                                dbg!(e);
                            }
                        }
                    });
                }
            });
            println!("Terminated");
        });
        let rt = helper::create_rt();
        Self {
            id_next: AtomicUsize::new(0),
            tx,
            rt,
            tasks: Mutex::new(RefCell::new(HashMap::new())),
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
        let tsk = Arc::new(Task::new(id, target, save_dir));
        self.rt
            .block_on(self.tx.send(Message::Job(tsk.clone())))
            .unwrap();

        self.tasks.lock().unwrap().borrow_mut().insert(id, tsk);
        id
    }

    /// Helper function to display progress
    /// todo
    pub fn state(&self, id: usize) -> String {
        println!("....");
        self.tasks.lock().unwrap().borrow()[&id].state()
    }

    /// Terminate the Downloader
    /// # Attention
    /// This will interrupt the downloading and leave the cached files undeleted.
    pub fn terminate(&self, id: usize) {
        self.rt.block_on(self.tx.send(Message::Terminate)).unwrap();
    }
}

impl Drop for Downloader {
    fn drop(&mut self) {
        if self.tx.is_closed() {
            return;
        }
        let tx_clone = self.tx.clone();
        self.rt.block_on(tx_clone.send(Message::Terminate)).unwrap();
        println!("Terminating")
    }
}

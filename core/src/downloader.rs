//! Downloader
//! Create a thread and spawn tasks through `run` on it, then run them asynchronously

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::executor::Executor;
use crate::task::Task;

#[derive(Debug)]
pub struct Downloader {
    id_next: AtomicUsize,
    exe: Arc<Executor>,
}

impl Downloader {
    /// Create a Downloader
    /// # Examples
    /// ```rust
    /// use core_api::downloader::Downloader;
    /// let dl = Downloader::new();
    /// ```
    pub fn new() -> Self {
        let exe = Arc::new(Executor::new());
        Self {
            id_next: AtomicUsize::new(0),
            exe,
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
        let task = Task::new(id, target, save_dir);
        self.exe.spawn_task(task);
        id
    }

    /// Helper function to display progress
    /// todo
    pub fn state(&self, id: usize) -> String {
        self.exe.process(id)
    }

    pub fn switch(&self, id: usize) {
        self.exe.switch(id);
    }

    /// Terminate the Downloader
    /// # Attention
    /// This will interrupt the downloading and leave the cached files undeleted.
    pub fn cancel(&self, id: usize) {
        self.exe.cancel(id);
    }

    pub fn terminate(&self) {
        self.exe.terminate();
    }
}

//! Downloader
//! Ask executor to control tasks

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::config;
use crate::executor::Executor;
use crate::task::Task;

#[derive(Debug)]
pub struct Downloader {
    id_next: AtomicUsize,
    exe: Arc<Executor>,
}

impl Downloader {
    /// Create a Downloader
    /// And use the config
    /// # Examples
    /// ```rust
    /// use core_api::downloader::Downloader;
    /// let dl = Downloader::new();
    /// ```
    pub fn new() -> Self {
        config::use_config();
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
    /// /* A cache dir will be made right beside the `save_dir`,
    /// and video will be saved at `save_dir`.
    /// The cache dir will be removed after finished */
    /// let id = dl.add_task(target);
    /// ```
    pub fn add_task(&self, target: String) -> usize {
        let id = self.id_next.fetch_add(1, Ordering::SeqCst);
        let task = Task::new(id, target);
        self.exe.spawn_task(task);
        id
    }

    pub fn title(&self, id: usize) -> String {
        self.exe.title(id)
    }

    pub fn process(&self, id: usize) -> String {
        self.exe.process(id)
    }

    pub fn state(&self, id: usize) -> usize {
        self.exe.state(id)
    }

    pub fn switch(&self, id: usize) {
        self.exe.switch(id);
    }

    pub fn cancel(&self, id: usize) {
        self.exe.cancel(id);
    }

    pub fn switch_all(&self) {
        self.exe.switch_all();
    }

    pub fn terminate(&self) {
        self.exe.terminate();
    }
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new()
    }
}

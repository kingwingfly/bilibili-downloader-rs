//! Downloader
//! Create a thread and spawn tasks through `run` on it, then run them asynchronously

use reqwest::{header, Client};
use std::sync::atomic::Ordering;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
// use std::sync::{atomic::AtomicUsize, Arc};
// use std::collections::HashMap;

use crate::config::{FINISHED, PARTS, TOTAL, USER_AGENT};
use crate::helper;
use crate::task::{Message, Task};

#[derive(Debug)]
pub struct Downloader {
    tx: mpsc::Sender<Message>,
    rt: tokio::runtime::Runtime,
    // jobs: Vec<Task>,
}


impl Downloader {
    /// Create a Downloader
    /// # Examples
    /// ```rust
    /// let dl = Downloader::new();
    /// ```
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::channel(8);
        std::thread::spawn(move || {
            let rt = helper::create_rt();
            rt.block_on(async move {
                while let Some(Message::Job(task)) = rx.recv().await {
                    dbg!(&task);
                    tokio::spawn(task.handle());
                }
            });
            println!("Terminated");
        });
        let rt = helper::create_rt();
        Self { tx, rt }
    }

    /// Run a downloading task
    /// # Examples
    /// ```rust
    /// use core_api::downloader::Downloader;
    /// let dl = Downloader::new();
    /// let target = "https://www.bilibili.com/video/BV1pm4y1B7Sp/?".to_owned();
    /// let save_path = "../tests/video_downloads".to_owned();
    /// /* A cache dir will be made right beside the `save_path`, 
    /// and video will be saved at `save_path`. 
    /// The cache dir will be removed when finished */
    /// dl.run(target, save_path);
    /// ```
    pub fn run(&self, target: String, save_path: String) {
        let (v_url, a_url, title) = self.parse(&target).unwrap();
        dbg!(&v_url, &a_url, &title);
        let v_path = format!("{save_path}/cache_{title}/{title}.mp4");
        let a_path = format!("{save_path}/cache_{title}/{title}.aac");
        let out_path = format!("{save_path}/{title}.mp4");
        self.download(v_path.clone(), v_url);
        self.download(a_path.clone(), a_url);
        self.rt.block_on(async move {
            loop {
                // dbg!(Self::state());
                if (FINISHED.load(Ordering::Relaxed) == TOTAL.load(Ordering::Relaxed))
                    && (TOTAL.load(Ordering::Relaxed) != 0)
                {
                    break;
                }
            }
            helper::merge(v_path, a_path, out_path).await.unwrap();
            helper::rm_cache(format!("{save_path}/cache_{title}/")).await;
        })
    }

    /// Spawn a task
    /// This is a helper function for `Downloader::download()`
    fn spawn_task(&self, id: usize, path: String, target: &str, range: String) {
        let tx = self.tx.clone();
        let msg = Message::Job(Task {
            id,
            path: path.to_owned(),
            target: target.into(),
            range,
            tx: None,
        });
        self.rt.block_on(async move {
            let path = std::path::PathBuf::from(path);
            let path = path.parent().unwrap();
            helper::mkdir(path).await;
            tx.send(msg).await.unwrap();
        });
    }

    /// A helper function for `Download::run()`
    /// Parse a video name
    /// Return the `video_url`, `audio_url` and `title`
    fn parse(&self, target: &str) -> Result<(String, String, String), Box<dyn std::error::Error>> {
        let tx_clone = self.tx.clone();
        self.rt.block_on(async move {
            let (tx, rx) = oneshot::channel();
            let tsk = Task {
                id: 0,
                path: "".into(),
                target: target.to_owned(),
                range: "".into(),
                tx: Some(tx),
            };
            tx_clone.send(Message::Job(tsk)).await.unwrap();
            let res = rx.await?;
            Ok(res)
        })
    }

    /// A helper function for `Download::run()`
    /// # Args
    /// `path`: Ends with `.mp4'
    /// `target`: A direct download url
    fn download(&self, path: String, target: String) {
        // get the content-length through a head request
        let total = self.rt.block_on(async {
            let resp = Client::new()
                .head(&target)
                .header(header::USER_AGENT, USER_AGENT)
                .header(header::REFERER, "https://www.bilibili.com/")
                .send()
                .await
                .unwrap();
            resp.headers()
                .get("content-length")
                .unwrap()
                .to_str()
                .unwrap()
                .parse::<usize>()
                .unwrap()
        });
        // refresh the workload
        TOTAL.fetch_add(total, Ordering::SeqCst);
        dbg!(&total);
        // divide the task into `PARTS` parts
        // and spawn them asynchronousl
        let quotient = total / PARTS;
        for id in 0..PARTS {
            let range = format!(
                "{}-{}",
                quotient * id,
                if id != (PARTS - 1) {
                    quotient * (id + 1) - 1
                } else {
                    total - 1
                }
            );
            self.spawn_task(id, path.clone(), &target, range)
        }
    }

    /// Helper function to display progress
    /// todo
    pub fn state() -> String {
        format!(
            "{}/{}",
            FINISHED.load(Ordering::Relaxed),
            TOTAL.load(Ordering::Relaxed)
        )
    }

    /// Terminate the Downloader
    /// # Attention
    /// This will interrupt the downloading and leave the cached files undeleted.
    pub async fn terminate(&self) {
        self.tx.send(Message::Terminate).await.unwrap();
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
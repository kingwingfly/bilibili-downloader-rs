use reqwest::{header, Client};
use std::convert::TryInto;
use std::io::SeekFrom;
use std::sync::Arc;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::sync::watch;

use crate::config::*;
use crate::controller::Controller;
use crate::helper;
use crate::process::Process;

type TaskResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
pub struct Task {
    pub id: usize,
    pub target: String,
    pub save_dir: String,
    pub process: Arc<Process>,
    pub ctl: Controller,
    pub rx: watch::Receiver<bool>,
}

impl Task {
    pub fn new(id: usize, target: String, save_dir: String) -> Self {
        let process = Arc::new(Process::new());
        let (tx, rx) = watch::channel(false);

        Self {
            id,
            target,
            save_dir,
            process,
            ctl: Controller::new(tx),
            rx,
        }
    }

    pub async fn execute(&self) -> TaskResult<()> {
        // todo use tokio::sync::Semaphore;
        let (v_url, a_url, title) = self.parse().await?;
        dbg!(&v_url, &a_url, &title);
        helper::mkdir(format!("{}/cache_{}/", self.save_dir, self.id)).await;
        let cache_path = |f| format!("{}/cache_{}/{title}.{f}", self.save_dir, self.id);
        let v_path = cache_path(VIDEO_FORMAT);
        let a_path = cache_path(AUDIO_FORMAT);
        let target_path = vec![(v_url, v_path.clone()), (a_url, a_path.clone())];
        self.download(target_path).await?;
        let out_path = format!("{}/{title}.{VIDEO_FORMAT}", self.save_dir);
        helper::merge(v_path, a_path, out_path).await.unwrap();
        self.rm_cache();
        Ok(())
    }

    /// A helper function for `Task::execute()`
    /// Parse a video name
    /// Return the `video_url`, `audio_url` and `title`
    async fn parse(&self) -> TaskResult<(String, String, String)> {
        let client = Client::new();
        let resp = client
            .get(&self.target)
            .header(header::COOKIE, COOKIE)
            // .header(header::ACCEPT, "text/html")
            .header(header::USER_AGENT, USER_AGENT)
            .send()
            .await?;
        let html = resp.text().await.unwrap();

        let re = regex::Regex::new(r#"\[\{"id":\d*,"baseUrl":"(.*?)""#).unwrap();
        let mut res = re.captures_iter(&html);
        let match_res = |cpt: regex::Captures| cpt.get(1).unwrap().as_str().to_owned();
        let v_url = match_res(res.next().unwrap());
        let a_url = match_res(res.next().unwrap());

        let re = regex::Regex::new(r#""videoData":\{.+?"title":"(.*?)",""#).unwrap();
        let title = match_res(re.captures(&html).unwrap());
        Ok((v_url, a_url, title))
    }

    /// A helper function for `Task::execute()`
    /// # Args
    /// `target`: A direct download url
    /// `path`: Ends with `VIDEO/AUDIO_FORMAT'
    async fn download(&self, target_path: Vec<(String, String)>) -> TaskResult<()> {
        let mut handles = Vec::new();
        for (target, path) in target_path {
            let total = Self::get_content_length(target.as_str()).await?;
            self.process.add_total(total);
            let quotient = total / PARTS;
            for i in 0..PARTS {
                let range = format!(
                    "{}-{}",
                    quotient * i,
                    if i != (PARTS - 1) {
                        quotient * (i + 1) - 1
                    } else {
                        total - 1
                    }
                );
                let rx = self.rx.clone();
                handles.push(tokio::spawn(Self::download_range(
                    target.to_owned(),
                    range,
                    path.to_owned(),
                    self.process.clone(),
                    rx,
                )));
            }
        }
        for handle in handles {
            handle.await??;
        }
        Ok(())
    }

    async fn download_range(
        target: String,
        range: String,
        path: String,
        process: Arc<Process>,
        mut rx: watch::Receiver<bool>,
    ) -> TaskResult<()> {
        let headers = Self::headers(range.as_str());
        dbg!(&headers);
        let client = Client::new();
        let mut resp = client.get(target).headers(headers).send().await?;
        let mut file = helper::fs_open(&path).await;
        let offset = range.split("-").next().unwrap().parse::<u64>().unwrap();
        file.seek(SeekFrom::Start(offset)).await.unwrap();
        loop {
            tokio::select! {
                Ok(Some(chunk)) = resp.chunk() => {
                    let size = chunk.len();
                    dbg!(&size);
                    file.write_all(&chunk).await.unwrap();
                    process.add_finished(size);
                }
                _ = async {}, if rx.has_changed().unwrap() => {
                    let _state = *rx.borrow_and_update();
                    let _state = rx.changed().await.is_ok();
                }
                else => break
            }
        }
        println!("finish range {}", range);
        Ok(())
    }

    pub fn switch(&self) {}

    async fn get_content_length(target: &str) -> TaskResult<usize> {
        Ok(Client::new()
            .head(target)
            .header(header::USER_AGENT, USER_AGENT)
            .header(header::REFERER, "https://www.bilibili.com/")
            .send()
            .await?
            .headers()
            .get("content-length")
            .unwrap()
            .to_str()?
            .parse::<usize>()?)
    }

    fn headers(range: &str) -> header::HeaderMap {
        let headers_json = format!(
            r#"
    {{
        "Sec-Fetch-Site": "cross-site",
        "Accept-Language": "zh-CN,zh-Hans;q=0.9",
        "Accept-Encoding": "identity",
        "Sec-Fetch-Mode": "cors",
        "Origin": "https://www.bilibili.com",
        "User-Agent": "{USER_AGENT}",
        "Referer": "https://www.bilibili.com/",
        "Range": "bytes={range}",
        "Sec-Fetch-Dest": "empty"
    }}"#
        );
        let hm: std::collections::HashMap<String, String> =
            serde_json::from_str(&headers_json).unwrap();
        let headers: header::HeaderMap = (&hm).try_into().unwrap();
        headers
    }

    pub fn rm_cache(&self) {
        helper::rm_cache(format!("{}/cache_{}/", self.save_dir, self.id));
    }

    pub fn process(&self) -> String {
        self.process.get()
    }
}

// #[derive(Debug)]
// pub struct TaskBuilder {
//     pub id: usize,
//     pub target: String,
//     pub save_dir: String,
//     pub process: Arc<Process>,
// }

// impl TaskBuilder {
//     pub fn new(id: usize, target: String, save_dir: String) -> Self {
//         let process = Arc::new(Process::new());
//         Self {
//             id,
//             target,
//             save_dir,
//             process,
//         }
//     }

//     pub fn build(&self) -> Task {
//         let (tx, rx) = watch::channel(false);
//         Task {
//             id: todo!(),
//             target: todo!(),
//             save_dir: todo!(),
//             process: todo!(),
//         }
//     }
// }

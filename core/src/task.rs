use reqwest::{header, Client};
use std::convert::TryInto;
use std::io::SeekFrom;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

use crate::config::*;
use crate::helper;
use crate::process::Process;

type TaskResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
pub(crate) enum Message {
    Job(Arc<Task>),
    Terminate,
}

#[derive(Debug)]
pub struct Task {
    pub id: usize,
    pub target: String,
    pub save_dir: String,
    pub process: Arc<Process>,
}

impl Task {
    pub fn new(id: usize, target: String, save_dir: String) -> Self {
        let process = Arc::new(Process::new());
        Self {
            id,
            target,
            save_dir,
            process,
        }
    }

    pub async fn execute(&self) -> TaskResult<()> {
        // todo use tokio::sync::Semaphore;
        let (v_url, a_url, title) = self.parse().await?;
        dbg!(&v_url, &a_url, &title);
        helper::mkdir(format!("{}/cache_{title}/", self.save_dir)).await;
        let cache_path = |f| format!("{}/cache_{title}/{title}.{f}", self.save_dir);
        let v_path = cache_path(VIDEO_FORMAT);
        let a_path = cache_path(AUDIO_FORMAT);
        self.download(v_url, v_path.clone()).await?;
        self.download(a_url, a_path.clone()).await?;
        // loop {
        //     // println!("{} process: {}", self.id, self.state());
        //     if self.process.finished() == self.process.total() && self.process.total() != 0 {
        //         break;
        //     }
        //     tokio::task::yield_now().await;
        // }
        let out_path = format!("{}/{title}.{VIDEO_FORMAT}", self.save_dir);
        helper::merge(v_path, a_path, out_path).await.unwrap();
        helper::rm_cache(format!("{}/cache_{title}/", self.save_dir)).await;
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
    async fn download(&self, target: String, path: String) -> TaskResult<()> {
        let total = Self::get_content_length(target.as_str()).await?;
        self.process.add_total(total);
        let mut handles = Vec::new();
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
            handles.push(tokio::spawn(Self::download_range(
                target.to_owned(),
                range,
                path.to_owned(),
                self.process.clone(),
            )));
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
    ) -> TaskResult<()> {
        let headers = Self::headers(range.as_str());
        dbg!(&headers);
        let client = Client::new();
        let mut resp = client.get(target).headers(headers).send().await?;
        let mut file = helper::fs_open(&path).await;
        let offset = range.split("-").next().unwrap().parse::<u64>().unwrap();
        file.seek(SeekFrom::Start(offset)).await.unwrap();
        // let mut temp = offset;
        while let Some(chunk) = resp.chunk().await.unwrap() {
            let size = chunk.len();
            // dbg!(&size);
            file.write_all(&chunk).await.unwrap();
            process.add_finished(size);
            // temp += size as u64;
            // println!("allocated between {}-{}", temp-size, temp);
        }
        println!("finish range {}", range);
        Ok(())
    }

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

    pub async fn cancel(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub fn state(&self) -> String {
        format!(
            "{}/{}",
            self.process.finished.load(Ordering::Relaxed),
            self.process.total.load(Ordering::Relaxed)
        )
    }
}

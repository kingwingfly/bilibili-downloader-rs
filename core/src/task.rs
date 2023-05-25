//! The task, including execute, operations and query functions.

use reqwest::{header, Client};
use std::cell::RefCell;
use std::io::SeekFrom;
use std::sync::Arc;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::sync::Mutex;
use tokio::task::JoinSet;

use crate::config::*;
use crate::headers::HeadersGen;
use crate::helper;
use crate::process::Process;
use crate::state::FSM;

type TaskResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
pub struct Task {
    pub id: usize,
    target: String,
    save_dir: String,
    title: Arc<Mutex<RefCell<String>>>,
    process: Arc<Process>,
    fsm: Arc<FSM>,
}

impl Task {
    pub fn new(id: usize, target: String) -> Self {
        let process = Arc::new(Process::new());
        Self {
            id,
            target,
            save_dir: SAVE_PATH.get().unwrap().to_owned(),
            title: Arc::new(Mutex::new(RefCell::new(String::new()))),
            process,
            fsm: Arc::new(FSM::new()),
        }
    }

    pub async fn execute(&self) -> TaskResult<()> {
        helper::mkdir(format!("{}/cache_{}/", self.save_dir, self.id)).await;
        let (v_url, a_url, title) = self.parse().await?;
        dbg!(&v_url, &a_url, &title);
        {
            let title_ = self.title.lock().await;
            title_.replace(title.clone());
        }
        let cache_path = |f| format!("{}/cache_{}/{title}.{f}", self.save_dir, self.id);
        let v_path = cache_path(VIDEO_FORMAT);
        let a_path = cache_path(AUDIO_FORMAT);
        let target_path = vec![(v_url, v_path.clone()), (a_url, a_path.clone())];
        let res = self.download(target_path).await?;
        match res {
            true => {
                let out_path = format!("{}/{title}.{VIDEO_FORMAT}", self.save_dir);
                helper::merge(v_path, a_path, out_path).await.unwrap();
                self.fsm.finish();
                println!("Task {} Finished", self.id);
            }
            false => {
                self.fsm.cancel();
                println!("Task {} Cancelled", self.id);
            }
        }
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
            .header(header::COOKIE, COOKIE.get().unwrap())
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
        let title = helper::file_name_filter(&match_res(re.captures(&html).unwrap()));
        Ok((v_url, a_url, title))
    }

    /// A helper function for `Task::execute()`
    /// # Args
    /// `target_path` is in the form of [(targte, path)]
    /// `target`: A direct download url
    /// `path`: Ends with `VIDEO/AUDIO_FORMAT'
    async fn download(&self, target_path: Vec<(String, String)>) -> TaskResult<bool> {
        let mut handles = JoinSet::new();
        for (target, path) in target_path {
            let total = Self::get_content_length(target.as_str()).await?;
            self.process.add_total(total);
            let headers_gen = Arc::new(HeadersGen::new(0, total));
            let file = Arc::new(helper::fs_open(&path).await);
            for _ in 0..*PARTS.get().unwrap() {
                let headers_gen_c = headers_gen.clone();
                let file_c = file.clone();
                handles.spawn(Self::download_range(
                    file_c,
                    target.to_owned(),
                    headers_gen_c,
                    self.process.clone(),
                    self.fsm.clone(),
                ));
            }
        }
        let mut res = true;
        for _ in 0..handles.len() {
            res &= handles.join_next().await.unwrap()??;
            if !res {
                break;
            }
        }
        Ok(res)
    } // JoinHandles are dropped with JoinSet here

    async fn download_range(
        file: Arc<tokio::sync::RwLock<tokio::io::BufWriter<tokio::fs::File>>>,
        target: String,
        headers_gen: Arc<HeadersGen>,
        process: Arc<Process>,
        fsm: Arc<FSM>,
    ) -> TaskResult<bool> {
        let client = Client::new();
        let res = loop {
            tokio::select! {
                Some(mut headers) = async { headers_gen.next() }, if fsm.now_state_code() == 0 => {
                    let mut offset = headers.get("Range").unwrap().to_str().unwrap().split('-').next().unwrap().split('=').last().unwrap().parse::<u64>().unwrap();
                    let mut file = file.write().await;
                    let mut resp = helper::get_resp(&client, &target, &headers).await;
                    file.seek(SeekFrom::Start(offset)).await.unwrap();
                    loop {
                        let gotten = resp.chunk().await;
                        match gotten {
                            Ok(Some(chunk)) => {
                                let size = chunk.len();
                                file.write_all(&chunk).await.unwrap();
                                process.add_finished(size);
                            },
                            Ok(None) => {break;},
                            Err(_) => {
                                println!("retry");
                                offset = file.seek(SeekFrom::Current(0)).await.unwrap();
                                let to = headers.get("Range").unwrap().to_str().unwrap().split('-').last().unwrap().parse::<usize>().unwrap();
                                headers.insert("Range", format!("bytes={offset}-{to}").parse().unwrap());
                                resp = helper::get_resp(&client, &target, &headers).await;
                                // println!("{headers:?}:{}\n", resp.status())
                            }
                        }
                    }
                }
                _ = async {}, if fsm.now_state_code() != 0 => {
                    let state_code = fsm.now_state_code();
                    match state_code {
                        2 => break false,
                        1 => {tokio::time::sleep(tokio::time::Duration::from_secs(1)).await},
                        _ => unreachable!(),
                    }
                }
                // true if no branch match, means succeed finishing
                else => break true
            }
        };
        Ok(res)
    }

    pub fn switch(&self) {
        self.fsm.switch();
    }

    pub fn cancel(&self) {
        self.fsm.cancel();
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

    fn rm_cache(&self) {
        helper::rm_cache(format!("{}/cache_{}/", self.save_dir, self.id));
    }

    pub fn title(&self) -> String {
        match self.title.try_lock() {
            Ok(title) => title.borrow().to_owned(),
            Err(_) => String::new(),
        }
    }

    pub fn process(&self) -> String {
        self.process.get()
    }

    pub fn state(&self) -> usize {
        self.fsm.now_state_code()
    }
}

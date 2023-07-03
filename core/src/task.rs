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
        let resp = Client::new()
            .get(target)
            .header(header::USER_AGENT, USER_AGENT)
            .header(header::REFERER, "https://www.bilibili.com/")
            .header(header::RANGE, "bytes=0-0")
            .send()
            .await?;
        let hd = resp.headers();
        dbg!(&hd);
        let length = hd
            .get("content-range")
            .unwrap_or(&header::HeaderValue::from_str(&format!("/{}", usize::MAX)).unwrap())
            .to_str()
            .unwrap()
            .split('/')
            .last()
            .unwrap()
            .parse::<usize>()
            .unwrap();
        Ok(length)
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_content_length() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let target = String::from("https://xy139x226x24x92xy.mcdn.bilivideo.cn:8082/v1/resource/1181828689-1-100110.m4s?agrr=0&build=0&buvid=&bvc=vod&bw=29918&cdnid=71704&deadline=1688362382&e=ig8euxZM2rNcNbdlhoNvNC8BqJIzNbfqXBvEqxTEto8BTrNvN0GvT90W5JZMkX_YN0MvXg8gNEV4NC8xNEV4N03eN0B5tZlqNxTEto8BTrNvNeZVuJ10Kj_g2UB02J0mN0B5tZlqNCNEto8BTrNvNC7MTX502C8f2jmMQJ6mqF2fka1mqx6gqj0eN0B599M%3D&gen=playurlv2&logo=80000000&mid=0&nbs=1&nettype=0&oi=2073295812&orderid=0%2C3&os=bcache&platform=pc&sign=de24e3&traceid=trosoULGtgptRC_0_e_N&uipk=5&uparams=e%2Cuipk%2Cnbs%2Cdeadline%2Cgen%2Cos%2Coi%2Ctrid%2Cmid%2Cplatform&upsig=2adf885b104fbd37e096b22dccb491c0");
            // let target = String::from("https://cn-jstz-cu-01-04.bilivideo.com/upgcxcode/89/86/1181828689/1181828689_nb3-1-30080.m4s?e=ig8euxZM2rNcNbdlhoNvNC8BqJIzNbfqXBvEqxTEto8BTrNvN0GvT90W5JZMkX_YN0MvXg8gNEV4NC8xNEV4N03eN0B5tZlqNxTEto8BTrNvNeZVuJ10Kj_g2UB02J0mN0B5tZlqNCNEto8BTrNvNC7MTX502C8f2jmMQJ6mqF2fka1mqx6gqj0eN0B599M=&uipk=5&nbs=1&deadline=1688360215&gen=playurlv2&os=bcache&oi=2073295812&trid=0000f176f753d0d145fcb3955155bed6c30eu&mid=32280488&platform=pc&upsig=ed7ade91c9210ac3fef1f0683b1123fe&uparams=e,uipk,nbs,deadline,gen,os,oi,trid,mid,platform&cdnid=71704&bvc=vod&nettype=0&orderid=0,3&buvid=1FF87ED7-2D57-84BE-3FD0-8F03EC5B47F949754infoc&build=0&agrr=0&bw=124636&logo=80000000");
            let length = Task::get_content_length(&target).await.unwrap();
            dbg!(length);
        })
    }
}

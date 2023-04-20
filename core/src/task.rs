use reqwest::{header, Client};
use std::convert::TryInto;
use std::io::SeekFrom;
use std::sync::atomic::Ordering;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::sync::oneshot;

use crate::config::{COOKIE, FINISHED, USER_AGENT};
use crate::helper;

#[derive(Debug)]
pub(crate) enum Message {
    Job(Task),
    Terminate,
}

#[derive(Debug)]
pub(crate) struct Task {
    pub id: usize,
    pub path: String,
    pub target: String,
    pub range: String,
    pub tx: Option<oneshot::Sender<(String, String, String)>>,
}

impl Task {
    pub async fn handle(self) {
        match self.tx {
            Some(_) => self.parse().await.unwrap(),
            None => self.download().await.unwrap(),
        };
    }

    async fn parse(self) -> Result<(), Box<dyn std::error::Error>> {
        let target = self.target;
        let client = Client::new();
        let resp = client
            .get(target)
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

        self.tx.map(|tx| tx.send((v_url, a_url, title)));
        Ok(())
    }

    async fn download(&self) -> Result<(), Box<dyn std::error::Error>> {
        let target = self.target.to_owned();
        let range = self.range.to_owned();
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
        dbg!(&headers);
        let client = Client::new();
        let mut resp = client.get(target).headers(headers).send().await?;
        dbg!(&resp.status());
        let mut file = helper::fs_open(&self.path).await;
        let offset = range.split("-").next().unwrap().parse::<u64>().unwrap();
        file.seek(SeekFrom::Start(offset)).await.unwrap();
        let mut temp = offset;
        while let Some(chunk) = resp.chunk().await.unwrap() {
            let size = chunk.len();
            // dbg!(&size);
            file.write_all(&chunk).await.unwrap();
            // println!("write from {}", self.id);
            FINISHED.fetch_add(size, Ordering::SeqCst);
            temp += size as u64;
        }
        println!("{} allocated at {}-{}", self.id, offset, temp);
        println!("finish");

        Ok(())
    }
}

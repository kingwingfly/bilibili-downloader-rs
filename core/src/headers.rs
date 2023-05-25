//! A header generation

use crate::config::{MINI_SIZE, USER_AGENT};
use reqwest::header;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub struct HeadersGen {
    start: AtomicUsize,
    to: AtomicUsize,
    end: usize,
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

impl HeadersGen {
    // input a range
    // return an iterator contains several headers
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start: AtomicUsize::new(start),
            to: AtomicUsize::new(std::cmp::min(start + MINI_SIZE, end)),
            end,
        }
    }

    pub fn next(&self) -> Option<header::HeaderMap> {
        let to = self.to.fetch_add(MINI_SIZE, Ordering::Relaxed);
        let start = self.start.swap(to + 1, Ordering::Relaxed);
        if start > self.end {
            return None;
        }
        let range = format!("{}-{}", start, std::cmp::min(to, self.end));
        let hm = headers(&range);
        Some(hm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let headers = std::sync::Arc::new(HeadersGen::new(0, 25_000_000_000_000));
        let mut jhs = Vec::new();
        let (tx, rx) = std::sync::mpsc::channel::<String>();
        let jh = std::thread::spawn(move || {
            let mut ranges = Vec::new();
            while let Ok(range) = rx.recv() {
                if ranges.contains(&range) {
                    return;
                }
                ranges.push(range.clone());
            }
        });
        for _ in 0..5 {
            let headers_c = headers.clone();
            let tx_c = tx.clone();
            let jh = std::thread::spawn(move || {
                while let Some(hm) = headers_c.next() {
                    let range = hm.get("Range").unwrap().to_str().unwrap();
                    println!("{}", range);
                    tx_c.send(range.to_string()).unwrap();
                }
            });
            jhs.push(jh);
        }
        let _ = jh.join();
        for jh in jhs {
            let _ = jh.join();
        }
    }
}

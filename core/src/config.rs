//! config
//! Maybe some config helper functions

use crate::helper;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::io::Write;

pub(crate) const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.5 Safari/605.1.15";
pub(crate) static COOKIE: OnceCell<String> = OnceCell::new();
pub(crate) static PARTS: OnceCell<usize> = OnceCell::new();
pub(crate) static SAVE_PATH: OnceCell<String> = OnceCell::new();
pub(crate) const VIDEO_FORMAT: &str = "mp4";
pub(crate) const AUDIO_FORMAT: &str = "aac";
// todo config

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    cookie: String,
    save_path: String,
    parts: usize,
}

impl Config {
    fn apply(&self) {
        COOKIE.set(self.cookie.to_owned()).unwrap();
        SAVE_PATH.set(self.save_path.to_owned()).unwrap();
        PARTS.set(self.parts).unwrap();
    }
}

pub fn use_config() {
    let config_path = helper::config_path();
    let file = std::fs::OpenOptions::new().read(true).open(config_path);
    match file {
        Ok(file) => {
            let config = serde_json::from_reader::<_, Config>(file).unwrap();
            config.apply();
        }
        Err(_) => {
            let config = Config {
                cookie: String::new(),
                save_path: helper::download_dir().to_str().unwrap().to_owned(),
                parts: 2,
            };
            config.apply();
        }
    }
}

pub fn submit_config(cookie: String, save_path: String, parts: usize) {
    let config_path = helper::config_path();
    let config = Config {
        cookie,
        save_path,
        parts,
    };
    let config_json = serde_json::to_string(&config).unwrap();
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(config_path)
        .unwrap();
    file.write_all(config_json.as_bytes()).unwrap();
}

pub fn read_config() -> (String, String, usize) {
    let config_path = helper::config_path();
    let file = std::fs::OpenOptions::new().read(true).open(config_path);
    match file {
        Ok(file) => {
            let config = serde_json::from_reader::<_, Config>(file).unwrap();
            (config.cookie, config.save_path, config.parts)
        }
        Err(_) => {
            let config = Config {
                cookie: String::new(),
                save_path: helper::download_dir().to_str().unwrap().to_owned(),
                parts: 2,
            };
            (config.cookie, config.save_path, config.parts)
        }
    }
}

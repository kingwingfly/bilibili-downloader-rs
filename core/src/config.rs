//! config
//! Maybe some config helper functions

use crate::helper;
use keyring::Entry;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::env;

pub(crate) const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.5 Safari/605.1.15";
pub(crate) static COOKIE: OnceCell<String> = OnceCell::new();
pub(crate) static PARTS: OnceCell<usize> = OnceCell::new();
pub(crate) static SAVE_PATH: OnceCell<String> = OnceCell::new();
pub(crate) static FFMPEG: OnceCell<String> = OnceCell::new();
pub(crate) static USER: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| match env::var("USERNAME") {
        Ok(user) => user,
        Err(_) => env::var("USER").unwrap(),
    });
pub(crate) const VIDEO_FORMAT: &str = "mp4";
pub(crate) const AUDIO_FORMAT: &str = "aac";

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    cookie: String,
    save_path: String,
    parts: usize,
    ffmpeg: String,
}

impl Config {
    fn apply(&self) {
        COOKIE.set(self.cookie.to_owned()).unwrap();
        SAVE_PATH.set(self.save_path.to_owned()).unwrap();
        PARTS.set(self.parts).unwrap();
        FFMPEG.set(self.ffmpeg.to_owned()).unwrap();
    }
}

pub fn use_config() {
    let config = Entry::new("bilibili downloader", &USER)
        .unwrap()
        .get_password();
    match &config {
        Ok(config) => {
            let config = serde_json::from_str::<Config>(config).unwrap();
            config.apply();
        }
        Err(_) => {
            let config = Config {
                cookie: String::new(),
                save_path: helper::download_dir().to_str().unwrap().to_owned(),
                parts: 1,
                ffmpeg: String::from("ffmpeg"),
            };
            config.apply();
        }
    }
}

pub fn submit_config(cookie: String, save_path: String, parts: usize, ffmpeg: String) {
    let config = Config {
        cookie,
        save_path,
        parts,
        ffmpeg,
    };
    let config_json = serde_json::to_string(&config).unwrap();
    let entry = Entry::new("bilibili downloader", &USER).unwrap();
    entry.set_password(&config_json).unwrap();
}

pub fn read_config() -> (String, String, usize, String) {
    let config = Entry::new("bilibili downloader", &USER)
        .unwrap()
        .get_password();
    let config = match &config {
        Ok(config) => serde_json::from_str::<Config>(config).unwrap(),
        Err(_) => Config {
            cookie: String::new(),
            save_path: helper::download_dir().to_str().unwrap().to_owned(),
            parts: 1,
            ffmpeg: String::from("ffmpeg"),
        },
    };
    (config.cookie, config.save_path, config.parts, config.ffmpeg)
}

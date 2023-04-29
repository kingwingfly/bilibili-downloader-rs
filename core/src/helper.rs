//! Helper funtions for bili_downlader

use tauri::api::path;
use tokio::fs::{self, OpenOptions};
use tokio::io::BufWriter;
use tokio::process::Command;

/// As the name, create a tokio runtime at current thread.
pub fn create_rt() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
}

/// Retuen a BufWriter, create new, overwrite but **no** truncate
pub async fn fs_open(path: &str) -> BufWriter<tokio::fs::File> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(path)
        .await
        .unwrap();
    BufWriter::new(file)
}

/// Create folders recursively
pub async fn mkdir<P: AsRef<std::path::Path>>(path: P) {
    fs::create_dir_all(path).await.unwrap();
}

/// Remove the cache folder
pub(crate) fn rm_cache<P: AsRef<std::path::Path>>(cache_path: P) {
    std::fs::remove_dir_all(cache_path).unwrap();
}

pub fn download_dir() -> std::path::PathBuf {
    path::download_dir().unwrap()
}

pub(crate) fn config_path() -> std::path::PathBuf {
    let mut config_path = path::config_dir().unwrap();
    config_path.push("bili_downloader");
    std::fs::create_dir_all(&config_path).unwrap();
    config_path.push("bili_downloader_config.json");
    config_path
}

/// Merge video and audio using this command:
/// `ffmpeg -y -i video.mp4 -i audio.mp4 -c:v copy -c:a copy -o output.mp4`
pub(crate) async fn merge(
    v_path: String,
    a_path: String,
    out_path: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let out_path = std::path::PathBuf::from(out_path);
    let out_dir = out_path.parent().unwrap();
    mkdir(out_dir).await;
    let output = Command::new("ffmpeg")
        .arg("-y")
        .arg("-i")
        .arg(v_path)
        .arg("-i")
        .arg(a_path)
        .arg("-c:v")
        .arg("copy")
        .arg("-c:a")
        .arg("copy")
        .arg(out_path.to_owned())
        .output()
        .await?;

    println!("status: {}", output.status);
    println!("stdout: {:?}", std::str::from_utf8(&output.stdout));
    println!("stderr: {:?}", std::str::from_utf8(&output.stderr));
    Ok(())
}

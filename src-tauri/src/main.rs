// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use core_api::{downloader::Downloader, helper};
use once_cell::sync::OnceCell;

static DOWNLOADER: OnceCell<Downloader> = OnceCell::new();

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn download(target: String, savedir: String) -> usize {
    let dl = DOWNLOADER.get_or_init(|| Downloader::new());
    dl.run(target, savedir)
}

#[tauri::command]
fn state(id: usize) -> String {
    DOWNLOADER.get().unwrap().state(id)
}

#[tauri::command]
fn terminate(id: usize) {
    DOWNLOADER.get().unwrap().terminate(id);
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![download, state, terminate])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

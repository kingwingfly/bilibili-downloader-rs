// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use core_api::{downloader::Downloader, helper};



// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn download(target: String, save_path: String) -> String {
    let dl = Downloader::new();
    dl.run(target, save_path);
    // todo return downloader id
}

#[tauri::command]
fn state(id: String) -> String {
    Downloader::state()
}

#[tauri::command]
fn terminate(id: String) {
    Downloader::state();
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![download, state, terminate])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

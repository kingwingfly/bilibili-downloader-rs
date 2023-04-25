// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use core_api::downloader::Downloader;
use once_cell::sync::OnceCell;

static DOWNLOADER: OnceCell<Downloader> = OnceCell::new();

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn add_task(target: String, savedir: String) -> usize {
    let dl = DOWNLOADER.get_or_init(|| Downloader::new());
    dl.add_task(target, savedir)
}

#[tauri::command]
fn process(id: usize) -> String {
    DOWNLOADER
        .get()
        .map_or_else(|| return String::new(), |dl| dl.process(id))
}

#[tauri::command]
fn state(id: usize) -> usize {
    DOWNLOADER
        .get()
        .map_or_else(|| return 404, |dl| dl.state(id))
    // 0 working; 1 pausing; 2 cancelled; 3 finished
}

#[tauri::command]
fn switch(id: usize) {
    DOWNLOADER
        .get()
        .map_or_else(|| return, |dl: &Downloader| dl.switch(id));
}

#[tauri::command]
fn cancel(id: usize) {
    DOWNLOADER.get().map_or_else(|| return, |dl| dl.cancel(id));
}

#[tauri::command]
fn switch_all() {
    DOWNLOADER
        .get()
        .map_or_else(|| return, |dl| dl.switch_all());
}

#[tauri::command]
fn terminate() {
    DOWNLOADER.get().map_or_else(|| return, |dl| dl.terminate());
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            add_task, process, state, switch, cancel, switch_all, terminate
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

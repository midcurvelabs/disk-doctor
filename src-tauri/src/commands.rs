use crate::{clean, disk, history, memory, scanner};
use std::process::Command;

#[tauri::command]
pub fn disk_status() -> disk::DiskStatus {
    disk::status()
}

#[tauri::command]
pub fn memory_pressure() -> memory::MemPressure {
    memory::pressure()
}

#[tauri::command]
pub async fn scan_all() -> Vec<scanner::CategoryResult> {
    tokio::task::spawn_blocking(scanner::scan_all)
        .await
        .unwrap_or_default()
}

#[tauri::command]
pub async fn scan_one(id: String) -> Option<scanner::CategoryResult> {
    tokio::task::spawn_blocking(move || scanner::scan_one(&id))
        .await
        .ok()
        .flatten()
}

#[tauri::command]
pub async fn clean(ids: Vec<String>, dry_run: bool) -> clean::CleanResult {
    tokio::task::spawn_blocking(move || clean::clean(&ids, dry_run))
        .await
        .unwrap_or(clean::CleanResult {
            recovered_bytes: 0,
            per_category: vec![],
        })
}

#[tauri::command]
pub fn history() -> Vec<history::CleanRecord> {
    history::load()
}

#[tauri::command]
pub fn open_in_finder(path: String) -> Result<(), String> {
    Command::new("open")
        .arg("-R")
        .arg(&path)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

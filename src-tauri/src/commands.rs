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
pub async fn top_processes(limit: usize) -> Vec<memory::ProcessRow> {
    tokio::task::spawn_blocking(move || memory::top_processes(limit))
        .await
        .unwrap_or_default()
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

#[tauri::command]
pub fn open_activity_monitor() -> Result<(), String> {
    Command::new("open")
        .args(["-a", "Activity Monitor"])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Polite quit (SIGTERM) for an aggregated row.
///
/// - `is_app_bundle = true`  → name is a parent .app bundle name. Every process
///                              whose exe path lives under `<name>.app/...` is
///                              signalled.
/// - `is_app_bundle = false` → name is a literal process name. Match exact.
///
/// We never SIGKILL. Apps get a chance to save state. We also refuse to signal
/// processes that map to a known critical macOS service (kernel_task etc.).
#[tauri::command]
pub fn quit_process(name: String, is_app_bundle: bool) -> Result<usize, String> {
    use sysinfo::{ProcessesToUpdate, Signal, System};

    const DO_NOT_KILL: &[&str] = &[
        "kernel_task",
        "launchd",
        "windowserver",
        "loginwindow",
        "coreaudiod",
        "cfprefsd",
        "fseventsd",
        "mds",
        "mds_stores",
        "powerd",
        "logd",
        "syslogd",
        "configd",
        "securityd",
        "trustd",
        "dock",
        "controlcenter",
        "systemuiserver",
    ];
    if !is_app_bundle && DO_NOT_KILL.contains(&name.to_lowercase().as_str()) {
        return Err(format!("Refusing to quit critical system process: {name}"));
    }

    let mut s = System::new();
    s.refresh_processes(ProcessesToUpdate::All, true);
    let mut sent = 0usize;
    for p in s.processes().values() {
        let matches = if is_app_bundle {
            p.exe()
                .map(|path| {
                    path.iter().any(|c| {
                        c.to_str()
                            .map(|s| s.strip_suffix(".app") == Some(&name))
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false)
        } else {
            p.name().to_string_lossy() == name
        };
        if matches && p.kill_with(Signal::Term).unwrap_or(false) {
            sent += 1;
        }
    }
    Ok(sent)
}

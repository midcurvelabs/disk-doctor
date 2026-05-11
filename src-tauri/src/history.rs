use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct CleanRecord {
    pub timestamp: String,
    pub category_ids: Vec<String>,
    pub recovered_bytes: u64,
}

fn history_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("no home dir"))?;
    let dir = home.join("Library/Application Support/disk-doctor");
    fs::create_dir_all(&dir)?;
    Ok(dir.join("history.json"))
}

pub fn load() -> Vec<CleanRecord> {
    let Ok(path) = history_path() else {
        return vec![];
    };
    let Ok(bytes) = fs::read(&path) else {
        return vec![];
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

pub fn record(ids: &[String], recovered_bytes: u64) -> Result<()> {
    let path = history_path()?;
    let mut log = load();
    log.push(CleanRecord {
        timestamp: Utc::now().to_rfc3339(),
        category_ids: ids.to_vec(),
        recovered_bytes,
    });
    fs::write(&path, serde_json::to_vec_pretty(&log)?)?;
    Ok(())
}

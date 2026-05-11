//! Walking + sizing logic for the catalog.

use crate::catalog::{CacheCategory, CATALOG};
use rayon::prelude::*;
use serde::Serialize;
use std::path::{Path, PathBuf};
use sysinfo::System;
use walkdir::WalkDir;

#[derive(Serialize, Clone)]
pub struct CategoryResult {
    pub id: String,
    pub label: String,
    pub tier: u8,
    pub size_bytes: u64,
    pub paths: Vec<String>,
    pub why: String,
    pub regenerates: String,
    pub running_app_block: Option<String>,
}

/// Resolve a path glob in the catalog into actual existing paths.
/// Supports literal paths and the simple `**` recursion used in our catalog.
pub fn resolve_paths(pat: &str) -> Vec<PathBuf> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return vec![],
    };
    let full = if pat.starts_with('/') {
        pat.to_string()
    } else {
        format!("{}/{}", home.display(), pat)
    };
    if full.contains('*') {
        match glob::glob(&full) {
            Ok(it) => it.filter_map(Result::ok).collect(),
            Err(_) => vec![],
        }
    } else {
        let p = PathBuf::from(&full);
        if p.exists() { vec![p] } else { vec![] }
    }
}

fn dir_size(root: &Path, max_depth: Option<usize>) -> u64 {
    let mut walker = WalkDir::new(root).follow_links(false);
    if let Some(d) = max_depth {
        walker = walker.max_depth(d);
    }
    walker
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

fn process_running(needle: &str) -> bool {
    let mut s = System::new();
    s.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let needle = needle.to_ascii_lowercase();
    s.processes()
        .values()
        .any(|p| p.name().to_string_lossy().to_ascii_lowercase().contains(&needle))
}

pub fn scan_category(cat: &CacheCategory) -> CategoryResult {
    let all_paths: Vec<PathBuf> = cat
        .paths
        .iter()
        .flat_map(|p| resolve_paths(p))
        .collect();

    let size: u64 = all_paths
        .par_iter()
        .map(|p| dir_size(p, cat.max_depth))
        .sum();

    let running_app_block = cat
        .running_check
        .and_then(|n| if process_running(n) { Some(n.to_string()) } else { None });

    CategoryResult {
        id: cat.id.to_string(),
        label: cat.label.to_string(),
        tier: cat.tier,
        size_bytes: size,
        paths: all_paths.iter().map(|p| p.display().to_string()).collect(),
        why: cat.why.to_string(),
        regenerates: cat.regenerates.to_string(),
        running_app_block,
    }
}

pub fn scan_all() -> Vec<CategoryResult> {
    CATALOG.par_iter().map(scan_category).collect()
}

pub fn scan_one(id: &str) -> Option<CategoryResult> {
    CATALOG.iter().find(|c| c.id == id).map(scan_category)
}

use crate::catalog::{CleanMethod, CATALOG};
use crate::history;
use crate::scanner;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize, Clone)]
pub struct PerCategory {
    pub id: String,
    pub recovered_bytes: u64,
    pub error: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct CleanResult {
    pub recovered_bytes: u64,
    pub per_category: Vec<PerCategory>,
}

fn remove_path(path: &Path, method: CleanMethod) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    match method {
        CleanMethod::Recursive | CleanMethod::TrashAll | CleanMethod::Trash => {
            // For v1 safety we always go via Trash on Tier 2+/anything moveable,
            // and we use recursive remove for Tier 1 (cache dirs we own).
            // Catalog encodes the intent; here we just dispatch.
            match method {
                CleanMethod::Recursive => {
                    if path.is_dir() {
                        std::fs::remove_dir_all(path).map_err(|e| e.to_string())
                    } else {
                        std::fs::remove_file(path).map_err(|e| e.to_string())
                    }
                }
                CleanMethod::Trash | CleanMethod::TrashAll => {
                    trash::delete(path).map_err(|e| e.to_string())
                }
            }
        }
    }
}

pub fn clean(ids: &[String], dry_run: bool) -> CleanResult {
    let mut per_category: Vec<PerCategory> = Vec::new();
    let mut total_recovered: u64 = 0;

    for id in ids {
        let cat = match CATALOG.iter().find(|c| &c.id == id) {
            Some(c) => c,
            None => {
                per_category.push(PerCategory {
                    id: id.clone(),
                    recovered_bytes: 0,
                    error: Some("unknown category".into()),
                });
                continue;
            }
        };

        // Refuse if owning app is running.
        let pre = scanner::scan_category(cat);
        if let Some(app) = &pre.running_app_block {
            per_category.push(PerCategory {
                id: id.clone(),
                recovered_bytes: 0,
                error: Some(format!("{app} is open — quit it first")),
            });
            continue;
        }

        let before_bytes = pre.size_bytes;
        if dry_run {
            per_category.push(PerCategory {
                id: id.clone(),
                recovered_bytes: before_bytes,
                error: None,
            });
            total_recovered += before_bytes;
            continue;
        }

        let mut err: Option<String> = None;
        for p_str in &pre.paths {
            let p = std::path::Path::new(p_str);
            if let Err(e) = remove_path(p, cat.method) {
                err = Some(e);
                break;
            }
        }

        let after_bytes = scanner::scan_category(cat).size_bytes;
        let recovered = before_bytes.saturating_sub(after_bytes);
        total_recovered += recovered;

        per_category.push(PerCategory {
            id: id.clone(),
            recovered_bytes: recovered,
            error: err,
        });
    }

    let result = CleanResult {
        recovered_bytes: total_recovered,
        per_category,
    };

    if !dry_run {
        let _ = history::record(ids, total_recovered);
    }

    result
}

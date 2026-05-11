use serde::Serialize;
use sysinfo::Disks;

#[derive(Serialize, Clone)]
pub struct DiskStatus {
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub used_bytes: u64,
    pub free_percent: f64,
}

/// Returns disk status for the user's data volume (/System/Volumes/Data on macOS).
pub fn status() -> DiskStatus {
    let disks = Disks::new_with_refreshed_list();
    // Prefer the data volume; fall back to the disk with the most space.
    let chosen = disks
        .iter()
        .find(|d| d.mount_point().to_string_lossy() == "/System/Volumes/Data")
        .or_else(|| disks.iter().max_by_key(|d| d.total_space()));
    if let Some(d) = chosen {
        let total = d.total_space();
        let free = d.available_space();
        let used = total.saturating_sub(free);
        let pct = if total == 0 {
            0.0
        } else {
            (free as f64 / total as f64) * 100.0
        };
        DiskStatus {
            total_bytes: total,
            free_bytes: free,
            used_bytes: used,
            free_percent: pct,
        }
    } else {
        DiskStatus {
            total_bytes: 0,
            free_bytes: 0,
            used_bytes: 0,
            free_percent: 0.0,
        }
    }
}

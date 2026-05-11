use serde::Serialize;
use sysinfo::System;

#[derive(Serialize, Clone)]
pub struct MemPressure {
    pub state: &'static str, // "green" | "amber" | "red"
    pub pages_free: u64,
    pub compressor_pages: u64,
    pub swap_used_bytes: u64,
}

/// Approximate macOS memory pressure using sysinfo: ratio of free RAM vs total,
/// combined with swap usage. This is a heuristic, not the precise jetsam signal.
pub fn pressure() -> MemPressure {
    let mut s = System::new_all();
    s.refresh_memory();
    let total = s.total_memory().max(1);
    let free = s.available_memory();
    let swap = s.used_swap();
    let free_pct = (free as f64 / total as f64) * 100.0;
    let state = if free_pct < 5.0 || swap > 4 * 1024 * 1024 * 1024 {
        "red"
    } else if free_pct < 15.0 || swap > 1024 * 1024 * 1024 {
        "amber"
    } else {
        "green"
    };
    MemPressure {
        state,
        pages_free: free / 4096,
        compressor_pages: 0,
        swap_used_bytes: swap,
    }
}

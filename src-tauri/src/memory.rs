use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use sysinfo::{ProcessesToUpdate, System};

#[derive(Serialize, Clone, Default)]
pub struct MemPressure {
    pub state: &'static str, // "green" | "amber" | "red"
    pub total_bytes: u64,
    pub app_bytes: u64,
    pub wired_bytes: u64,
    pub compressed_bytes: u64,
    pub cached_files_bytes: u64,
    pub available_bytes: u64,
    pub swap_used_bytes: u64,
    pub swap_total_bytes: u64,
}

#[derive(Serialize, Clone)]
pub struct ProcessRow {
    pub pid: u32,
    /// Display name. For app-bundle processes this is the parent .app name
    /// (e.g. "Dia", "Claude"). For CLI tools, the binary name.
    pub name: String,
    /// Sum of resident memory across all member processes for app-bundle rows.
    pub rss_bytes: u64,
    /// How many member processes were aggregated into this row.
    pub process_count: u32,
    /// "Safe" | "Risky" | "DoNotKill"
    pub safety: &'static str,
    /// Human-readable explanation of `safety` + what this process actually is.
    pub safety_reason: String,
    /// True if this row represents an aggregated set of processes from one .app
    /// bundle. False for standalone CLI tools / system processes.
    pub is_app_bundle: bool,
}

pub fn pressure() -> MemPressure {
    if let Some(p) = pressure_via_vm_stat() {
        return p;
    }
    pressure_via_sysinfo()
}

fn pressure_via_vm_stat() -> Option<MemPressure> {
    let out = Command::new("vm_stat").output().ok()?;
    if !out.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut page_size: u64 = 4096;
    let mut stats: HashMap<String, u64> = HashMap::new();
    for line in stdout.lines() {
        if line.contains("page size of") {
            if let Some(rest) = line.split("page size of").nth(1) {
                let n: String = rest
                    .chars()
                    .skip_while(|c| !c.is_ascii_digit())
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                if let Ok(ps) = n.parse::<u64>() {
                    if ps > 0 {
                        page_size = ps;
                    }
                }
            }
            continue;
        }
        if let Some((k, v)) = line.split_once(':') {
            let val: String = v.trim().trim_end_matches('.').to_string();
            if let Ok(pages) = val.parse::<u64>() {
                stats.insert(k.trim().to_string(), pages.saturating_mul(page_size));
            }
        }
    }
    let total_ram = sysinfo_total_ram();
    let wired = *stats.get("Pages wired down").unwrap_or(&0);
    let active = *stats.get("Pages active").unwrap_or(&0);
    let inactive = *stats.get("Pages inactive").unwrap_or(&0);
    let speculative = *stats.get("Pages speculative").unwrap_or(&0);
    let purgeable = *stats.get("Pages purgeable").unwrap_or(&0);
    let free = *stats.get("Pages free").unwrap_or(&0);
    let compressed = *stats
        .get("Pages occupied by compressor")
        .or_else(|| stats.get("Pages stored in compressor"))
        .unwrap_or(&0);
    let app = active;
    let cached_files = inactive.saturating_add(speculative).saturating_add(purgeable);
    let available = free.saturating_add(cached_files);
    let swap = swap_via_sysctl();
    let state = pressure_state(total_ram, available, swap.0);
    Some(MemPressure {
        state,
        total_bytes: total_ram,
        app_bytes: app,
        wired_bytes: wired,
        compressed_bytes: compressed,
        cached_files_bytes: cached_files,
        available_bytes: available,
        swap_used_bytes: swap.0,
        swap_total_bytes: swap.1,
    })
}

fn pressure_via_sysinfo() -> MemPressure {
    let mut s = System::new();
    s.refresh_memory();
    let total = s.total_memory().max(1);
    let available = s.available_memory();
    let swap = (s.used_swap(), s.total_swap());
    MemPressure {
        state: pressure_state(total, available, swap.0),
        total_bytes: total,
        app_bytes: total.saturating_sub(available).saturating_sub(swap.0 / 4),
        wired_bytes: 0,
        compressed_bytes: 0,
        cached_files_bytes: 0,
        available_bytes: available,
        swap_used_bytes: swap.0,
        swap_total_bytes: swap.1,
    }
}

fn pressure_state(total: u64, available: u64, swap_used: u64) -> &'static str {
    let avail_pct = (available as f64 / total.max(1) as f64) * 100.0;
    if avail_pct < 5.0 || swap_used > 4 * 1024 * 1024 * 1024 {
        "red"
    } else if avail_pct < 15.0 || swap_used > 1024 * 1024 * 1024 {
        "amber"
    } else {
        "green"
    }
}

fn sysinfo_total_ram() -> u64 {
    let mut s = System::new();
    s.refresh_memory();
    s.total_memory()
}

fn swap_via_sysctl() -> (u64, u64) {
    let out = match Command::new("sysctl").arg("-n").arg("vm.swapusage").output() {
        Ok(o) => o,
        Err(_) => return (0, 0),
    };
    let s = String::from_utf8_lossy(&out.stdout);
    fn pick(s: &str, key: &str) -> u64 {
        let Some(idx) = s.find(key) else { return 0 };
        let rest = &s[idx + key.len()..];
        let trimmed = rest.trim_start_matches(|c: char| c == '=' || c.is_whitespace());
        let mut num = String::new();
        let mut chars = trimmed.chars();
        while let Some(c) = chars.next() {
            if c.is_ascii_digit() || c == '.' {
                num.push(c);
            } else {
                let n: f64 = num.parse().unwrap_or(0.0);
                let mult: f64 = match c {
                    'K' => 1024.0,
                    'M' => 1024.0 * 1024.0,
                    'G' => 1024.0 * 1024.0 * 1024.0,
                    _ => 1.0,
                };
                return (n * mult) as u64;
            }
        }
        0
    }
    (pick(&s, "used"), pick(&s, "total"))
}

// ----------------- Process classification -----------------

/// Critical macOS processes — quitting them will destabilize the session.
/// Names lowercased for matching against process name.
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
    "mdworker_shared",
    "mdworker",
    "powerd",
    "logd",
    "syslogd",
    "notifyd",
    "diskarbitrationd",
    "configd",
    "securityd",
    "trustd",
    "amfid",
    "syspolicyd",
    "tccd",
    "useractivityd",
    "dock",
    "controlcenter",
    "systemuiserver",
    "spotlight",
    "spotlightagent",
    "spotlightnetworkhelper",
    "knowledge-agent",
    "wallpaperagent",
    "lsd",
    "translationd",
];

/// Dev tools that the user is likely actively using — safe but worth warning.
const DEV_TOOLS: &[&str] = &[
    "node",
    "next",
    "vite",
    "esbuild",
    "tsc",
    "python",
    "python3",
    "ruby",
    "go",
    "rustc",
    "cargo",
    "claude",
    "codex",
    "ollama",
    "ollama-runner",
];

/// Walk a binary path to find the outermost `.app` bundle. Helper processes
/// like `/Applications/Dia.app/Contents/Frameworks/Browser Helper.app/Contents/MacOS/Browser Helper`
/// resolve to "Dia" because that's the first `.app` from root.
fn parent_app_from_path(path: &Path) -> Option<String> {
    for part in path.iter() {
        let s = part.to_str()?;
        if let Some(stripped) = s.strip_suffix(".app") {
            return Some(stripped.to_string());
        }
    }
    None
}

#[derive(Clone)]
struct Classification {
    safety: &'static str,
    reason: String,
}

fn classify(
    name: &str,
    path: &Path,
    parent_app: Option<&str>,
    process_count: u32,
) -> Classification {
    let path_str = path.to_string_lossy().to_lowercase();
    let name_lower = name.to_lowercase();

    if DO_NOT_KILL.iter().any(|c| name_lower == *c) {
        return Classification {
            safety: "DoNotKill",
            reason: format!(
                "{name} is a macOS system process. Quitting it will destabilize your session — leave it alone."
            ),
        };
    }

    if let Some(app) = parent_app {
        // User-facing app from /Applications. Tell the user what happens if they quit.
        let extra = if process_count > 1 {
            format!(
                " {} is running {process_count} processes — Disk Doctor will quit them all.",
                app
            )
        } else {
            String::new()
        };
        return Classification {
            safety: "Safe",
            reason: format!(
                "{app} is an installed app. Safe to quit — you may lose unsaved work in it.{extra}"
            ),
        };
    }

    if DEV_TOOLS.iter().any(|t| name_lower == *t) {
        return Classification {
            safety: "Safe",
            reason: format!(
                "{name} is a developer tool. Safe to quit — but it'll stop whatever it's running (dev server, model, build). Only quit if you're not actively using it."
            ),
        };
    }

    if path_str.starts_with("/system/") || path_str.starts_with("/usr/libexec/") {
        return Classification {
            safety: "Risky",
            reason: format!(
                "{name} is an Apple system process. It's likely fine to leave alone — quit only if you know what it does."
            ),
        };
    }

    if path_str.starts_with("/library/") {
        return Classification {
            safety: "Risky",
            reason: format!(
                "{name} is a system-wide helper (e.g. an installed daemon). Quitting may affect background services."
            ),
        };
    }

    Classification {
        safety: "Risky",
        reason: format!(
            "{name} has no obvious parent app. Open Activity Monitor and look at its parent before quitting."
        ),
    }
}

/// Top processes by resident memory.
///
/// Aggregation strategy:
///   - Processes living inside a `.app` bundle are summed under their parent app name.
///   - Everything else (CLI tools, daemons) is shown standalone, deduped by name.
pub fn top_processes(limit: usize) -> Vec<ProcessRow> {
    let mut s = System::new();
    s.refresh_processes(ProcessesToUpdate::All, true);

    struct Agg {
        rss: u64,
        first_pid: u32,
        count: u32,
        sample_path: std::path::PathBuf,
        is_app_bundle: bool,
        display_name: String,
    }
    let mut groups: HashMap<String, Agg> = HashMap::new();

    for (pid, p) in s.processes() {
        let rss = p.memory();
        if rss == 0 {
            continue;
        }
        let pid_u32 = pid.as_u32();
        let exe = p.exe().map(|p| p.to_path_buf()).unwrap_or_default();
        let raw_name = p.name().to_string_lossy().to_string();
        let parent_app = parent_app_from_path(&exe);

        // Group key: parent app if present, otherwise process name.
        let (key, display, is_app) = match &parent_app {
            Some(app) => (format!("app::{app}"), app.clone(), true),
            None => (format!("name::{raw_name}"), raw_name.clone(), false),
        };

        groups
            .entry(key)
            .and_modify(|a| {
                a.rss = a.rss.saturating_add(rss);
                a.count += 1;
                if pid_u32 < a.first_pid {
                    a.first_pid = pid_u32;
                }
            })
            .or_insert(Agg {
                rss,
                first_pid: pid_u32,
                count: 1,
                sample_path: exe.clone(),
                is_app_bundle: is_app,
                display_name: display,
            });
    }

    let mut rows: Vec<ProcessRow> = groups
        .into_values()
        .map(|a| {
            let parent_app = if a.is_app_bundle {
                Some(a.display_name.as_str())
            } else {
                None
            };
            let cls = classify(
                &a.display_name,
                &a.sample_path,
                parent_app,
                a.count,
            );
            ProcessRow {
                pid: a.first_pid,
                name: a.display_name,
                rss_bytes: a.rss,
                process_count: a.count,
                safety: cls.safety,
                safety_reason: cls.reason,
                is_app_bundle: a.is_app_bundle,
            }
        })
        .collect();
    rows.sort_by(|a, b| b.rss_bytes.cmp(&a.rss_bytes));
    rows.truncate(limit);
    rows
}

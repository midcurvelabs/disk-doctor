//! The Pipeline tab: leftovers from Rik's own video workflow.
//!
//! The cache catalog knows generic app caches. This module knows a workflow:
//! edit intermediates of finished shorts, raw takes and renders that belong on
//! the backup drive, render temp folders, old review boards. Rules live in a
//! user-editable JSON file (`pipeline.json` next to `history.json`), seeded with
//! defaults on first run, so a new pipeline folder is one JSON entry, not code.
//!
//! Safety contract for this tab:
//! 1. Only items that pass their gate are offered: the short is finished in its
//!    run-state file, or nothing in the item changed for `min_age_days`.
//! 2. `trash` rules move files to the Trash (reversible).
//! 3. `offload` rules never remove a file until a byte-identical copy (sha256)
//!    exists on the backup drive, either one found there already or one we just
//!    copied and verified. Then the local file goes to the Trash.
//! 4. Every run writes a history record before returning.

use anyhow::{anyhow, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use walkdir::WalkDir;

// ====================================================================
// Config
// ====================================================================

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    /// Move to the Trash. For files that regenerate or are no longer needed.
    Trash,
    /// Copy to the backup drive, verify sha256, then move the local file to the Trash.
    Offload,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Gate {
    /// Item is eligible when its slug is finished in a run-state file.
    #[serde(default)]
    pub done: bool,
    /// Item is eligible when nothing in it changed for this many days.
    #[serde(default)]
    pub min_age_days: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Rule {
    pub id: String,
    pub label: String,
    pub why: String,
    /// 1 = safe, 2 = confirm, 3 = your call (same meaning as the Disk tab).
    pub tier: u8,
    pub action: Action,
    /// Folder that holds the items. Relative to $HOME unless it starts with '/'.
    pub root: String,
    /// Glob (relative to root) that selects the items, e.g. "*" or "[0-9]*".
    pub items: String,
    /// Item names to skip (globs on the name).
    #[serde(default)]
    pub exclude_items: Vec<String>,
    /// File globs relative to the item. `{item}` is replaced by the item name.
    /// `**/*` means the whole item.
    pub files: Vec<String>,
    /// File globs (relative to the item) to keep.
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub gate: Gate,
    /// Only items owned by the current user (for shared folders like /private/tmp).
    #[serde(default)]
    pub own_only: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Backup {
    /// Mount point of the backup drive.
    pub root: String,
    /// Folder on the drive that new offloads go to: <root>/<offload_dir>/<rule>/<item>/<file>.
    pub offload_dir: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub version: u32,
    /// Warn when free space drops below this many GB.
    pub threshold_gb: u64,
    /// Minimum hours between two low-space notifications.
    pub notify_every_hours: u64,
    pub backup: Backup,
    /// Run-state files (relative to $HOME) whose entries carry a `stage`.
    pub runstate_globs: Vec<String>,
    /// Stages that mean a short is finished.
    pub done_stages: Vec<String>,
    pub rules: Vec<Rule>,
}

fn rule(
    id: &str,
    label: &str,
    why: &str,
    tier: u8,
    action: Action,
    root: &str,
    items: &str,
    files: &[&str],
    exclude: &[&str],
    gate: Gate,
) -> Rule {
    Rule {
        id: id.into(),
        label: label.into(),
        why: why.into(),
        tier,
        action,
        root: root.into(),
        items: items.into(),
        exclude_items: vec![],
        files: files.iter().map(|s| s.to_string()).collect(),
        exclude: exclude.iter().map(|s| s.to_string()).collect(),
        gate,
        own_only: false,
    }
}

impl Default for Config {
    fn default() -> Self {
        let rec = "Documents/rik-code/projects/video-engine/shorts/video-editor/studio/recordings";
        let done_or = |days| Gate { done: true, min_age_days: Some(days) };
        let age = |days| Gate { done: false, min_age_days: Some(days) };
        let mut tmp = rule(
            "render-temp",
            "Render temp folders",
            "Scratch folders that render and capture jobs leave in /private/tmp.",
            1,
            Action::Trash,
            "/private/tmp",
            "*",
            &["**/*"],
            &[],
            age(2),
        );
        tmp.own_only = true;
        tmp.exclude_items = ["claude-*", "cc-*", "com.*", "codex-*", "*.sock", "clang-module-cache", "launch-*", "tmux-*"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let short_renders = rule(
            "short-renders",
            "Render versions of finished shorts",
            "Every render version and motion render kept per short in rik-media. Finals stay.",
            2,
            Action::Offload,
            "Documents/rik-media/RikGPT/shorts",
            "[0-9]*",
            &["renders/**/*.mp4", "renders/**/*.mov"],
            &[],
            done_or(14),
        );
        Config {
            version: 1,
            threshold_gb: 30,
            notify_every_hours: 6,
            backup: Backup {
                root: "/Volumes/SandiskSSD-Media".into(),
                offload_dir: "2026/_disk-doctor-offload".into(),
            },
            runstate_globs: vec!["Documents/rik-docs/02_projects/RikGPT/shorts/_system/runs/batch-*.json".into()],
            done_stages: ["delivered", "scheduled", "published", "posted"].iter().map(|s| s.to_string()).collect(),
            rules: vec![
                rule(
                    "shorts-intermediates",
                    "Edit intermediates of finished shorts",
                    "CFR copies, matte and base renders, voice wavs. The edit specs (json) stay.",
                    1,
                    Action::Trash,
                    rec,
                    "*",
                    &["edit/**/*.mp4", "edit/**/*.mov", "edit/**/*.wav", "edit/**/*.m4a", "edit/**/*.webm"],
                    &["edit/{item}.mp4"],
                    done_or(21),
                ),
                tmp,
                rule(
                    "shorts-takes",
                    "Raw takes and cut reels of finished shorts",
                    "Camera takes and the cut reel. Copied to the SanDisk and verified before removal.",
                    2,
                    Action::Offload,
                    rec,
                    "*",
                    &["take-*.mp4", "take-*.mov", "edit/{item}.mp4"],
                    &[],
                    done_or(21),
                ),
                short_renders,
                rule(
                    "motion-renders",
                    "Motion piece renders",
                    "Rendered MP4s of motion pieces. Sources stay; the renders live in the SanDisk motion library.",
                    2,
                    Action::Offload,
                    "Documents/rik-code/projects/video-engine/motion",
                    "*",
                    &["*.mp4", "_v*/**/*.mp4", "renders/**/*.mp4"],
                    &[],
                    age(10),
                ),
                rule(
                    "desk-boards",
                    "Old Clip Desk boards",
                    "Frame boards of review rounds that are closed.",
                    2,
                    Action::Trash,
                    "Documents/rik-media/RikGPT/shorts/_desk",
                    "*",
                    &["**/*"],
                    &[],
                    age(10),
                ),
                rule(
                    "codex-sessions",
                    "Old Codex session logs",
                    "Codex keeps every session transcript forever. Days older than a month.",
                    3,
                    Action::Trash,
                    ".codex/sessions",
                    "*/*/*",
                    &["**/*"],
                    &[],
                    age(30),
                ),
            ],
        }
    }
}

pub fn app_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("no home dir"))?;
    let dir = home.join("Library/Application Support/disk-doctor");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn config_path() -> Result<PathBuf> {
    Ok(app_dir()?.join("pipeline.json"))
}

/// Load the rules file, seeding it with the defaults on first run.
pub fn load_config() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        let cfg = Config::default();
        fs::write(&path, serde_json::to_vec_pretty(&cfg)?)?;
        return Ok(cfg);
    }
    let bytes = fs::read(&path)?;
    serde_json::from_slice(&bytes).map_err(|e| anyhow!("pipeline.json: {e}"))
}

pub fn save_config(cfg: &Config) -> Result<()> {
    fs::write(config_path()?, serde_json::to_vec_pretty(cfg)?)?;
    Ok(())
}

fn expand(p: &str) -> PathBuf {
    if p.starts_with('/') {
        PathBuf::from(p)
    } else {
        dirs::home_dir().unwrap_or_default().join(p)
    }
}

// ====================================================================
// Scan
// ====================================================================

#[derive(Serialize, Clone, Debug)]
pub struct Item {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub file_count: usize,
    pub eligible: bool,
    pub reason: String,
    #[serde(skip)]
    pub files: Vec<(PathBuf, u64)>,
    /// The whole item goes (files == ["**/*"]): trash the folder, not each file.
    #[serde(skip)]
    pub whole: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct RuleResult {
    pub id: String,
    pub label: String,
    pub why: String,
    pub tier: u8,
    pub action: Action,
    pub root: String,
    /// Bytes that pass the gate and can go now.
    pub ready_bytes: u64,
    /// Bytes that match but are not finished or old enough yet.
    pub waiting_bytes: u64,
    pub items: Vec<Item>,
    pub error: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct BackupStatus {
    pub root: String,
    pub mounted: bool,
    pub free_bytes: u64,
}

#[derive(Serialize, Clone, Debug)]
pub struct Scan {
    pub rules: Vec<RuleResult>,
    pub backup: BackupStatus,
    pub trash_bytes: u64,
    pub threshold_gb: u64,
    pub config_path: String,
}

/// Slugs of shorts whose stage is one of `done_stages` in any run-state file.
pub fn done_slugs(cfg: &Config) -> HashSet<String> {
    let mut out = HashSet::new();
    let done: HashSet<&str> = cfg.done_stages.iter().map(String::as_str).collect();
    for pat in &cfg.runstate_globs {
        let full = expand(pat);
        let Ok(paths) = glob::glob(&full.to_string_lossy()) else { continue };
        for p in paths.flatten() {
            let Ok(bytes) = fs::read(&p) else { continue };
            let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes) else { continue };
            let Some(obj) = v.as_object() else { continue };
            for (slug, entry) in obj {
                if let Some(stage) = entry.get("stage").and_then(|s| s.as_str()) {
                    if done.contains(stage) {
                        out.insert(slug.clone());
                    }
                }
            }
        }
    }
    out
}

/// "68-foo--alt" belongs to short "68-foo".
fn slug_of(name: &str) -> &str {
    name.split("--").next().unwrap_or(name)
}

fn pattern(p: &str) -> Option<glob::Pattern> {
    glob::Pattern::new(p).ok()
}

fn match_opts() -> glob::MatchOptions {
    glob::MatchOptions { case_sensitive: true, require_literal_separator: true, require_literal_leading_dot: false }
}

fn days_since(t: SystemTime) -> f64 {
    SystemTime::now().duration_since(t).unwrap_or(Duration::ZERO).as_secs_f64() / 86_400.0
}

fn collect_files(rule: &Rule, item_path: &Path, name: &str) -> (Vec<(PathBuf, u64)>, Option<SystemTime>) {
    let mut seen: HashSet<PathBuf> = HashSet::new();
    let mut files: Vec<(PathBuf, u64)> = Vec::new();
    let mut newest: Option<SystemTime> = None;
    let excludes: Vec<glob::Pattern> = rule.exclude.iter().filter_map(|e| pattern(&e.replace("{item}", name))).collect();

    if item_path.is_file() {
        // A loose file at item level only counts when a file glob names it.
        let fname = item_path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let named = rule.files.iter().any(|f| {
            let f = f.replace("{item}", name);
            f == "**/*" || (!f.contains('/') && pattern(&f).map_or(false, |p| p.matches(&fname)))
        });
        if !named {
            return (files, newest);
        }
        if let Ok(m) = fs::symlink_metadata(item_path) {
            if m.file_type().is_file() {
                newest = m.modified().ok();
                files.push((item_path.to_path_buf(), m.len()));
            }
        }
        return (files, newest);
    }

    // Age is judged on everything in the item, not only the matched files,
    // so an item someone is still working on never looks old.
    for e in WalkDir::new(item_path).follow_links(false).into_iter().filter_map(Result::ok) {
        if let Ok(m) = e.metadata() {
            if let Ok(t) = m.modified() {
                newest = Some(newest.map_or(t, |n| n.max(t)));
            }
        }
    }

    for fpat in &rule.files {
        let full = item_path.join(fpat.replace("{item}", name));
        let Ok(paths) = glob::glob_with(&full.to_string_lossy(), match_opts()) else { continue };
        for p in paths.flatten() {
            let Ok(m) = fs::symlink_metadata(&p) else { continue };
            if !m.file_type().is_file() {
                continue;
            }
            let rel = p.strip_prefix(item_path).unwrap_or(&p);
            if excludes.iter().any(|x| x.matches_path_with(rel, match_opts())) {
                continue;
            }
            if seen.insert(p.clone()) {
                files.push((p, m.len()));
            }
        }
    }
    (files, newest)
}

pub fn scan_rule(rule: &Rule, done: &HashSet<String>) -> RuleResult {
    let root = expand(&rule.root);
    let mut res = RuleResult {
        id: rule.id.clone(),
        label: rule.label.clone(),
        why: rule.why.clone(),
        tier: rule.tier,
        action: rule.action,
        root: root.display().to_string(),
        ready_bytes: 0,
        waiting_bytes: 0,
        items: vec![],
        error: None,
    };
    if !root.is_dir() {
        return res;
    }
    let item_glob = root.join(&rule.items);
    let paths: Vec<PathBuf> = match glob::glob_with(&item_glob.to_string_lossy(), match_opts()) {
        Ok(it) => it.flatten().collect(),
        Err(e) => {
            res.error = Some(format!("bad items glob: {e}"));
            return res;
        }
    };
    let skip: Vec<glob::Pattern> = rule.exclude_items.iter().filter_map(|p| pattern(p)).collect();
    let my_uid = fs::metadata(dirs::home_dir().unwrap_or_default()).map(|m| m.uid()).unwrap_or(u32::MAX);
    let whole = rule.files.len() == 1 && rule.files[0] == "**/*" && rule.exclude.is_empty();

    let mut items: Vec<Item> = paths
        .par_iter()
        .filter_map(|p| {
            // Nested item globs ("*/*/*") show the path under root, e.g. "2026/10/02".
            let name = if rule.items.contains('/') {
                p.strip_prefix(&root).ok()?.to_string_lossy().to_string()
            } else {
                p.file_name()?.to_string_lossy().to_string()
            };
            let leaf = p.file_name()?.to_string_lossy().to_string();
            if skip.iter().any(|s| s.matches(&leaf)) {
                return None;
            }
            let meta = fs::symlink_metadata(p).ok()?;
            if meta.file_type().is_symlink() {
                return None;
            }
            if rule.own_only && meta.uid() != my_uid {
                return None;
            }
            let (files, newest) = collect_files(rule, p, &name);
            if files.is_empty() {
                return None;
            }
            let size: u64 = files.iter().map(|(_, s)| s).sum();
            let finished = rule.gate.done && done.contains(slug_of(&name));
            let age = newest.map(days_since).unwrap_or(0.0);
            let old = rule.gate.min_age_days.map_or(false, |d| age >= d as f64);
            let (eligible, reason) = if finished {
                (true, "finished in run state".to_string())
            } else if old {
                (true, format!("untouched for {:.0} days", age.floor()))
            } else if rule.gate.done {
                match rule.gate.min_age_days {
                    Some(d) => (false, format!("not finished; changed {:.0} days ago (waits {d})", age.floor())),
                    None => (false, "not finished in run state".into()),
                }
            } else {
                (false, format!("changed {:.0} days ago (waits {})", age.floor(), rule.gate.min_age_days.unwrap_or(0)))
            };
            Some(Item {
                name,
                path: p.display().to_string(),
                size_bytes: size,
                file_count: files.len(),
                eligible,
                reason,
                files,
                whole: whole && p.is_dir(),
            })
        })
        .collect();

    items.sort_by(|a, b| b.eligible.cmp(&a.eligible).then(b.size_bytes.cmp(&a.size_bytes)));
    res.ready_bytes = items.iter().filter(|i| i.eligible).map(|i| i.size_bytes).sum();
    res.waiting_bytes = items.iter().filter(|i| !i.eligible).map(|i| i.size_bytes).sum();
    res.items = items;
    res
}

fn dir_bytes(root: &Path) -> u64 {
    WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

pub fn backup_status(cfg: &Config) -> BackupStatus {
    let root = PathBuf::from(&cfg.backup.root);
    let mounted = root.is_dir() && fs::read_dir(&root).is_ok();
    let free_bytes = if mounted {
        sysinfo::Disks::new_with_refreshed_list()
            .iter()
            .find(|d| d.mount_point() == root.as_path())
            .map(|d| d.available_space())
            .unwrap_or(0)
    } else {
        0
    };
    BackupStatus { root: cfg.backup.root.clone(), mounted, free_bytes }
}

pub fn trash_bytes() -> u64 {
    dirs::home_dir().map(|h| dir_bytes(&h.join(".Trash"))).unwrap_or(0)
}

pub fn scan_with(cfg: &Config) -> Scan {
    let done = done_slugs(cfg);
    let rules = cfg.rules.iter().map(|r| scan_rule(r, &done)).collect();
    Scan {
        rules,
        backup: backup_status(cfg),
        trash_bytes: trash_bytes(),
        threshold_gb: cfg.threshold_gb,
        config_path: config_path().map(|p| p.display().to_string()).unwrap_or_default(),
    }
}

pub fn scan() -> Result<Scan> {
    Ok(scan_with(&load_config()?))
}

// ====================================================================
// Clean and offload
// ====================================================================

#[derive(Serialize, Clone, Debug, Default)]
pub struct Progress {
    pub phase: String,
    pub done_bytes: u64,
    pub total_bytes: u64,
    pub current: String,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct RuleOutcome {
    pub id: String,
    /// Bytes moved to the Trash.
    pub trashed_bytes: u64,
    /// Bytes copied to the backup drive (verified).
    pub copied_bytes: u64,
    /// Bytes that already had a verified copy on the drive.
    pub already_backed_bytes: u64,
    pub files: usize,
    pub errors: Vec<String>,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct RunResult {
    pub dry_run: bool,
    pub trashed_bytes: u64,
    pub copied_bytes: u64,
    pub rules: Vec<RuleOutcome>,
    pub trash_bytes_after: u64,
}

/// sha256 of a file, cached by (path, size, mtime) so the backup drive is not
/// re-read on every run.
pub struct HashCache {
    path: PathBuf,
    map: HashMap<String, String>,
    dirty: bool,
}

impl HashCache {
    pub fn load() -> Self {
        let path = app_dir().map(|d| d.join("hash-cache.json")).unwrap_or_default();
        let map = fs::read(&path).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
        HashCache { path, map, dirty: false }
    }

    fn key(p: &Path) -> Option<String> {
        let m = fs::metadata(p).ok()?;
        let mt = m.modified().ok()?.duration_since(SystemTime::UNIX_EPOCH).ok()?.as_secs();
        Some(format!("{}|{}|{}", p.display(), m.len(), mt))
    }

    pub fn sha256(&mut self, p: &Path, on_bytes: &mut dyn FnMut(u64)) -> Result<String> {
        let key = Self::key(p);
        if let Some(k) = &key {
            if let Some(h) = self.map.get(k) {
                return Ok(h.clone());
            }
        }
        let mut f = fs::File::open(p)?;
        let mut h = Sha256::new();
        let mut buf = vec![0u8; 4 << 20];
        loop {
            let n = f.read(&mut buf)?;
            if n == 0 {
                break;
            }
            h.update(&buf[..n]);
            on_bytes(n as u64);
        }
        let hex = hex::encode(h.finalize());
        if let Some(k) = key {
            self.map.insert(k, hex.clone());
            self.dirty = true;
        }
        Ok(hex)
    }

    pub fn save(&self) {
        if self.dirty {
            if let Ok(b) = serde_json::to_vec(&self.map) {
                let _ = fs::write(&self.path, b);
            }
        }
    }
}

/// Size index of every file on the backup drive (hidden system folders skipped).
fn index_backup(root: &Path) -> HashMap<u64, Vec<PathBuf>> {
    let mut idx: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    let walker = WalkDir::new(root).follow_links(false).into_iter().filter_entry(|e| {
        let n = e.file_name().to_string_lossy();
        e.depth() == 0 || !n.starts_with('.')
    });
    for e in walker.filter_map(Result::ok) {
        if !e.file_type().is_file() {
            continue;
        }
        if let Ok(m) = e.metadata() {
            if m.len() > 0 {
                idx.entry(m.len()).or_default().push(e.into_path());
            }
        }
    }
    idx
}

fn copy_file(src: &Path, dst: &Path, on_bytes: &mut dyn FnMut(u64)) -> Result<()> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = dst.with_extension(format!(
        "{}.ddpart",
        dst.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default()
    ));
    {
        let mut r = fs::File::open(src)?;
        let mut w = fs::File::create(&tmp)?;
        let mut buf = vec![0u8; 4 << 20];
        loop {
            let n = r.read(&mut buf)?;
            if n == 0 {
                break;
            }
            w.write_all(&buf[..n])?;
            on_bytes(n as u64);
        }
        w.sync_all()?;
    }
    fs::rename(&tmp, dst)?;
    Ok(())
}

/// A destination path that does not overwrite anything already on the drive.
fn free_dest(dst: PathBuf) -> PathBuf {
    if !dst.exists() {
        return dst;
    }
    let stem = dst.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let ext = dst.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    for n in 2.. {
        let cand = dst.with_file_name(format!("{stem}__{n}{ext}"));
        if !cand.exists() {
            return cand;
        }
    }
    unreachable!()
}

/// Run the selected rules. `dry_run` reports what would happen without touching
/// anything (offload dry runs do not read the backup drive).
pub fn run(rule_ids: &[String], dry_run: bool, progress: &mut dyn FnMut(Progress)) -> Result<RunResult> {
    run_with(&load_config()?, rule_ids, dry_run, progress)
}

pub fn run_with(cfg: &Config, rule_ids: &[String], dry_run: bool, progress: &mut dyn FnMut(Progress)) -> Result<RunResult> {
    let done = done_slugs(cfg);
    let mut result = RunResult { dry_run, ..Default::default() };

    let selected: Vec<(&Rule, RuleResult)> = cfg
        .rules
        .iter()
        .filter(|r| rule_ids.contains(&r.id))
        .map(|r| (r, scan_rule(r, &done)))
        .collect();

    let needs_backup = selected.iter().any(|(r, s)| r.action == Action::Offload && s.ready_bytes > 0);
    let backup_root = PathBuf::from(&cfg.backup.root);
    let mounted = backup_status(cfg).mounted;

    let total: u64 = selected.iter().map(|(_, s)| s.ready_bytes).sum();
    let mut done_bytes = 0u64;

    let mut index: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    if needs_backup && mounted && !dry_run {
        progress(Progress { phase: "Indexing the backup drive".into(), done_bytes, total_bytes: total, current: cfg.backup.root.clone() });
        index = index_backup(&backup_root);
    }
    let mut hashes = HashCache::load();

    for (rule, scan) in &selected {
        let mut out = RuleOutcome { id: rule.id.clone(), ..Default::default() };
        if rule.action == Action::Offload && !mounted {
            if scan.ready_bytes > 0 {
                out.errors.push(format!("{} is not connected", cfg.backup.root));
            }
            result.rules.push(out);
            continue;
        }
        for item in scan.items.iter().filter(|i| i.eligible) {
            if dry_run {
                out.files += item.file_count;
                match rule.action {
                    Action::Trash => out.trashed_bytes += item.size_bytes,
                    Action::Offload => {
                        out.copied_bytes += item.size_bytes;
                        out.trashed_bytes += item.size_bytes;
                    }
                }
                continue;
            }
            match rule.action {
                Action::Trash => {
                    progress(Progress { phase: "Moving to Trash".into(), done_bytes, total_bytes: total, current: item.name.clone() });
                    if item.whole {
                        match trash::delete(&item.path) {
                            Ok(()) => {
                                out.trashed_bytes += item.size_bytes;
                                out.files += item.file_count;
                            }
                            Err(e) => out.errors.push(format!("{}: {e}", item.name)),
                        }
                    } else {
                        for (f, size) in &item.files {
                            match trash::delete(f) {
                                Ok(()) => {
                                    out.trashed_bytes += size;
                                    out.files += 1;
                                }
                                Err(e) => out.errors.push(format!("{}: {e}", f.display())),
                            }
                        }
                    }
                    done_bytes += item.size_bytes;
                }
                Action::Offload => {
                    for (f, size) in &item.files {
                        let label = f.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                        let base = done_bytes;
                        let tick = |phase: &str, n: u64, progress: &mut dyn FnMut(Progress)| {
                            progress(Progress { phase: phase.into(), done_bytes: base + n.min(*size), total_bytes: total, current: label.clone() });
                        };
                        match offload_file(f, *size, rule, item, cfg, &backup_root, &mut index, &mut hashes, &mut |phase, n| tick(phase, n, progress)) {
                            Ok(OffloadKind::AlreadyThere) => {
                                out.already_backed_bytes += size;
                                out.trashed_bytes += size;
                                out.files += 1;
                            }
                            Ok(OffloadKind::Copied) => {
                                out.copied_bytes += size;
                                out.trashed_bytes += size;
                                out.files += 1;
                            }
                            Err(e) => out.errors.push(format!("{}: {e}", f.display())),
                        }
                        done_bytes += size;
                    }
                }
            }
        }
        result.trashed_bytes += out.trashed_bytes;
        result.copied_bytes += out.copied_bytes;
        result.rules.push(out);
    }
    hashes.save();

    if !dry_run {
        let ids: Vec<String> = rule_ids.iter().map(|i| format!("pipeline:{i}")).collect();
        let _ = crate::history::record(&ids, result.trashed_bytes);
    }
    result.trash_bytes_after = trash_bytes();
    Ok(result)
}

enum OffloadKind {
    AlreadyThere,
    Copied,
}

#[allow(clippy::too_many_arguments)]
fn offload_file(
    f: &Path,
    size: u64,
    rule: &Rule,
    item: &Item,
    cfg: &Config,
    backup_root: &Path,
    index: &mut HashMap<u64, Vec<PathBuf>>,
    hashes: &mut HashCache,
    tick: &mut dyn FnMut(&str, u64),
) -> Result<OffloadKind> {
    let mut read = 0u64;
    let local = hashes.sha256(f, &mut |n| {
        read += n;
        tick("Hashing", read / 3);
    })?;

    // 1. A byte-identical copy already on the drive? Then only trash the local one.
    if let Some(cands) = index.get(&size).cloned() {
        for c in cands {
            if let Ok(h) = hashes.sha256(&c, &mut |_| {}) {
                if h == local {
                    trash::delete(f).map_err(|e| anyhow!("trash: {e}"))?;
                    return Ok(OffloadKind::AlreadyThere);
                }
            }
        }
    }

    // 2. Copy, verify, then trash.
    let item_dir = Path::new(&item.path);
    let rel = if item_dir.is_dir() { f.strip_prefix(item_dir).unwrap_or(f).to_path_buf() } else { PathBuf::from(f.file_name().unwrap_or_default()) };
    let dst = free_dest(backup_root.join(&cfg.backup.offload_dir).join(&rule.id).join(&item.name).join(rel));
    let mut copied = 0u64;
    copy_file(f, &dst, &mut |n| {
        copied += n;
        tick("Copying to backup", size / 3 + copied / 3);
    })?;
    let mut verified = 0u64;
    let remote = hashes.sha256(&dst, &mut |n| {
        verified += n;
        tick("Verifying copy", 2 * size / 3 + verified / 3);
    })?;
    if remote != local {
        return Err(anyhow!("copy on the backup drive does not match (kept local file)"));
    }
    index.entry(size).or_default().push(dst);
    trash::delete(f).map_err(|e| anyhow!("trash: {e}"))?;
    Ok(OffloadKind::Copied)
}

/// Deletes everything inside ~/.Trash (the folder itself stays). Returns bytes freed.
pub fn empty_trash() -> Result<u64> {
    let trash = dirs::home_dir().ok_or_else(|| anyhow!("no home dir"))?.join(".Trash");
    let before = dir_bytes(&trash);
    let entries = fs::read_dir(&trash).map_err(|e| anyhow!("cannot read the Trash ({e}); give Disk Doctor Full Disk Access"))?;
    let mut first_err: Option<String> = None;
    for e in entries.flatten() {
        let p = e.path();
        let r = match fs::symlink_metadata(&p) {
            Ok(m) if m.is_dir() => fs::remove_dir_all(&p),
            Ok(_) => fs::remove_file(&p),
            Err(e) => Err(e),
        };
        if let Err(e) = r {
            first_err.get_or_insert(format!("{}: {e}", p.display()));
        }
    }
    let freed = before.saturating_sub(dir_bytes(&trash));
    let _ = crate::history::record(&["pipeline:empty-trash".to_string()], freed);
    match first_err {
        Some(e) if freed == 0 => Err(anyhow!(e)),
        _ => Ok(freed),
    }
}

// ====================================================================
// Watcher helpers
// ====================================================================

/// Bytes ready across all rules (cheap enough for a 10 minute background check).
pub fn ready_bytes() -> u64 {
    scan().map(|s| s.rules.iter().map(|r| r.ready_bytes).sum()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_strips_alt_suffix() {
        assert_eq!(slug_of("68-grok-bot--alt"), "68-grok-bot");
        assert_eq!(slug_of("51-opus"), "51-opus");
    }

    #[test]
    fn gate_and_excludes() {
        let base = std::env::temp_dir().join(format!("dd-pipeline-test-{}", std::process::id()));
        let item = base.join("07-demo");
        fs::create_dir_all(item.join("edit")).unwrap();
        fs::write(item.join("edit/src_cfr.mp4"), b"abc").unwrap();
        fs::write(item.join("edit/07-demo.mp4"), b"final").unwrap();
        fs::write(item.join("edit.json"), b"{}").unwrap();
        let r = Rule {
            id: "t".into(),
            label: "t".into(),
            why: "t".into(),
            tier: 1,
            action: Action::Trash,
            root: base.display().to_string(),
            items: "*".into(),
            exclude_items: vec![],
            files: vec!["edit/**/*.mp4".into()],
            exclude: vec!["edit/{item}.mp4".into()],
            gate: Gate { done: true, min_age_days: Some(30) },
            own_only: false,
        };
        let mut done = HashSet::new();
        let res = scan_rule(&r, &done);
        assert_eq!(res.items.len(), 1);
        assert!(!res.items[0].eligible, "fresh and unfinished must wait");
        assert_eq!(res.items[0].file_count, 1, "final cut and json are kept");
        done.insert("07-demo".to_string());
        let res = scan_rule(&r, &done);
        assert!(res.items[0].eligible);
        assert_eq!(res.ready_bytes, 3);
        fs::remove_dir_all(&base).unwrap();
    }

    /// End to end: one file already on the "drive" (only trashed), one new file
    /// (copied, verified, trashed), and the run never touches a fresh item.
    #[test]
    fn offload_copies_verifies_and_trashes() {
        let base = std::env::temp_dir().join(format!("dd-offload-test-{}", std::process::id()));
        let drive = base.join("drive");
        let item = base.join("src/09-demo");
        fs::create_dir_all(&item).unwrap();
        fs::create_dir_all(drive.join("old")).unwrap();
        let tag = std::process::id();
        let known = format!("dd-known-{tag}.bin");
        let fresh = format!("dd-fresh-{tag}.bin");
        fs::write(item.join(&known), b"already backed up").unwrap();
        fs::write(drive.join("old/elsewhere.bin"), b"already backed up").unwrap();
        fs::write(item.join(&fresh), b"brand new bytes").unwrap();

        let mut cfg = Config::default();
        cfg.backup = Backup { root: drive.display().to_string(), offload_dir: "off".into() };
        cfg.runstate_globs = vec![];
        cfg.rules = vec![Rule {
            id: "r".into(),
            label: "r".into(),
            why: "r".into(),
            tier: 2,
            action: Action::Offload,
            root: base.join("src").display().to_string(),
            items: "*".into(),
            exclude_items: vec![],
            files: vec!["*.bin".into()],
            exclude: vec![],
            gate: Gate { done: false, min_age_days: Some(0) },
            own_only: false,
        }];
        // backup_status needs a mount point match for free bytes only; mounted = is_dir.
        let res = run_with(&cfg, &["r".to_string()], false, &mut |_| {}).unwrap();
        let out = &res.rules[0];
        assert!(out.errors.is_empty(), "{:?}", out.errors);
        assert_eq!(out.already_backed_bytes, 17);
        assert_eq!(out.copied_bytes, 15);
        assert!(!item.join(&known).exists() && !item.join(&fresh).exists());
        let copy = drive.join("off/r/09-demo").join(&fresh);
        assert_eq!(fs::read(&copy).unwrap(), b"brand new bytes");
        // Tidy the two test files out of the real Trash.
        if let Some(h) = dirs::home_dir() {
            for e in fs::read_dir(h.join(".Trash")).into_iter().flatten().flatten() {
                let n = e.file_name().to_string_lossy().to_string();
                if n.starts_with(&format!("dd-known-{tag}")) || n.starts_with(&format!("dd-fresh-{tag}")) {
                    let _ = fs::remove_file(e.path());
                }
            }
        }
        fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn default_config_roundtrips() {
        let cfg = Config::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let back: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(back.rules.len(), cfg.rules.len());
    }
}

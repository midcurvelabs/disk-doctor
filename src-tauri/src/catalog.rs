//! The cache catalog — the heart of the app.
//!
//! Each entry describes a known cache pattern, its tier (1=safe, 2=confirm,
//! 3=user-call), what app owns it (so we can refuse to clean while open), and
//! the human explanation that shows in the UI.
//!
//! Adding a category = one entry in this file. No new Rust code anywhere else.

use once_cell::sync::Lazy;
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub enum CleanMethod {
    /// Plain recursive remove. Used for cache dirs we definitely own.
    Recursive,
    /// Move to system Trash via the `trash` crate. Safer; reversible.
    Trash,
    /// Each found path is wiped via Trash. Used for scan-many-find-many cases
    /// (e.g. node_modules across many projects).
    TrashAll,
}

#[derive(Clone, Debug, Serialize)]
pub struct CacheCategory {
    pub id: &'static str,
    pub label: &'static str,
    pub tier: u8,
    /// Globs relative to $HOME unless they start with '/'.
    pub paths: &'static [&'static str],
    /// Scan ceiling for walking — protects against runaway sizes.
    pub max_depth: Option<usize>,
    pub why: &'static str,
    pub regenerates: &'static str,
    /// Process name (lowercase substring) to block cleaning while running.
    pub running_check: Option<&'static str>,
    pub method: CleanMethod,
}

pub static CATALOG: Lazy<Vec<CacheCategory>> = Lazy::new(|| {
    use CleanMethod::*;
    vec![
        CacheCategory {
            id: "npm-cache",
            label: "npm package cache",
            tier: 1,
            paths: &[".npm/_cacache", ".npm/_npx"],
            max_depth: None,
            why: "Every npm package version you've ever installed sits here in case you reinstall it.",
            regenerates: "npm rebuilds the cache on next install.",
            running_check: None,
            method: Recursive,
        },
        CacheCategory {
            id: "node-modules",
            label: "node_modules across all projects",
            tier: 1,
            paths: &["Documents/rik-code/**/node_modules", "Documents/Personal/**/node_modules"],
            max_depth: Some(8),
            why: "400–1100 MB per project. Worktrees duplicate them.",
            regenerates: "Run `pnpm install` or `npm install` per project to restore.",
            running_check: None,
            method: TrashAll,
        },
        CacheCategory {
            id: "opencode-snapshots",
            label: "opencode session snapshots",
            tier: 1,
            paths: &[".local/share/opencode/snapshot"],
            max_depth: None,
            why: "Full file-system snapshots per opencode session for rollback.",
            regenerates: "Recreated as new sessions run.",
            running_check: Some("opencode"),
            method: Recursive,
        },
        CacheCategory {
            id: "codex-archived",
            label: "Codex archived sessions + logs DB",
            tier: 1,
            paths: &[".codex/archived_sessions", ".codex/logs_2.sqlite", ".codex/logs_2.sqlite-wal"],
            max_depth: None,
            why: "Old Codex CLI session archives + a fat logs SQLite.",
            regenerates: "Codex writes new sessions/logs as you use it.",
            running_check: Some("codex"),
            method: Recursive,
        },
        CacheCategory {
            id: "claude-vm",
            label: "Claude Code agent VM disk",
            tier: 2,
            paths: &["Library/Application Support/Claude/vm_bundles/claudevm.bundle"],
            max_depth: None,
            why: "A full Linux virtual-machine disk image that Claude's agent SDK runs in.",
            regenerates: "Claude rebuilds it on next agent run (slow first launch, then fine).",
            running_check: Some("claude"),
            method: Recursive,
        },
        CacheCategory {
            id: "ollama-models",
            label: "Ollama local models",
            tier: 2,
            paths: &[".ollama/models"],
            max_depth: None,
            why: "Local LLMs you downloaded but probably forgot about.",
            regenerates: "`ollama pull <model>` re-downloads any you want back.",
            running_check: Some("ollama"),
            method: Recursive,
        },
        CacheCategory {
            id: "xcode-derived",
            label: "Xcode DerivedData + Archives",
            tier: 1,
            paths: &[
                "Library/Developer/Xcode/DerivedData",
                "Library/Developer/Xcode/Archives",
                "Library/Developer/Xcode/iOS DeviceSupport",
                "Library/Developer/Xcode/watchOS DeviceSupport",
            ],
            max_depth: None,
            why: "Xcode build artifacts and device-symbol caches.",
            regenerates: "Regenerates on next build / device connect.",
            running_check: Some("xcode"),
            method: Recursive,
        },
        CacheCategory {
            id: "xcode-simulators",
            label: "iOS Simulators",
            tier: 2,
            paths: &["Library/Developer/CoreSimulator/Devices"],
            max_depth: None,
            why: "Snapshots of simulated iPhones/iPads. Often unused.",
            regenerates: "Recreate via Xcode > Settings > Platforms.",
            running_check: Some("simulator"),
            method: Recursive,
        },
        CacheCategory {
            id: "descript-partitions",
            label: "Descript electron partitions",
            tier: 2,
            paths: &["Library/Application Support/Descript/Partitions"],
            max_depth: None,
            why: "Descript embeds a full Chromium per editor partition.",
            regenerates: "Rebuilds on next launch.",
            running_check: Some("descript"),
            method: Recursive,
        },
        CacheCategory {
            id: "electron-cache-notion",
            label: "Notion / Slack / Discord / Stremio caches",
            tier: 2,
            paths: &[
                "Library/Application Support/Notion/Partitions",
                "Library/Application Support/Slack/Cache",
                "Library/Application Support/Slack/Service Worker",
                "Library/Application Support/discord/Cache",
                "Library/Application Support/stremio-server/stremio-cache",
            ],
            max_depth: None,
            why: "Electron apps each ship a Chromium and cache the web at 4K speed.",
            regenerates: "All rebuild on next app launch.",
            running_check: None,
            method: Recursive,
        },
        CacheCategory {
            id: "browser-caches",
            label: "Browser caches (Chrome / Brave / Dia)",
            tier: 1,
            paths: &[
                "Library/Caches/Google",
                "Library/Caches/BraveSoftware",
                "Library/Caches/Dia",
            ],
            max_depth: None,
            why: "Chromium-flavor caches. Distinct from your bookmarks/logins.",
            regenerates: "Refills as you browse.",
            running_check: None,
            method: Recursive,
        },
        CacheCategory {
            id: "pip-cache",
            label: "pip package cache",
            tier: 1,
            paths: &["Library/Caches/pip"],
            max_depth: None,
            why: "Python package wheels cache.",
            regenerates: "Refills on next pip install.",
            running_check: None,
            method: Recursive,
        },
        CacheCategory {
            id: "playwright",
            label: "Playwright browser binaries",
            tier: 1,
            paths: &["Library/Caches/ms-playwright", "Library/Caches/ms-playwright-go"],
            max_depth: None,
            why: "Headless Chromium/Firefox/WebKit downloaded by Playwright.",
            regenerates: "Re-downloads when you next install Playwright deps.",
            running_check: None,
            method: Recursive,
        },
        CacheCategory {
            id: "user-logs",
            label: "User logs (CreativeCloud, Adobe, Claude, NGL, …)",
            tier: 1,
            paths: &["Library/Logs"],
            max_depth: Some(2),
            why: "App debug logs you almost never read.",
            regenerates: "Apps write new logs as needed.",
            running_check: None,
            method: Recursive,
        },
        CacheCategory {
            id: "trash",
            label: "Empty Trash",
            tier: 1,
            paths: &[".Trash"],
            max_depth: Some(1),
            why: "The terminal trash dump.",
            regenerates: "—",
            running_check: None,
            method: Recursive,
        },
    ]
});

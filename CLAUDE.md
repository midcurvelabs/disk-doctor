# Disk Doctor — CLAUDE.md

## Project
Ship Season 01, Day 6. Tauri 2 + Rust + React menubar app for macOS that audits and cleans agent-era developer cruft (Claude VMs, opencode snapshots, node_modules across many projects, Ollama models, Electron partitions, Xcode caches, browser caches). Spec lives at `~/Documents/rik-docs/02_projects/midcurved-media/ship-season-01/builds/06-disk-doctor/prd.md` — read it first.

## Stack (locked, do not change without asking)
- Tauri 2.x · Rust 1.78+ · pnpm
- React 18 · TypeScript 5 · Vite 5
- Tailwind v4 · shadcn-style components · Lucide icons
- Zustand for frontend state
- JSON file at `~/Library/Application Support/disk-doctor/history.json`
- macOS 13+ universal (arm64 + x86_64)

## What we are NOT building (NON-GOALS for v1)
- Windows or Linux support
- Cleaning anything outside `~` (no `/Library`, no `/private/var`, no system caches). Exception (Rik, 24 Oct 2026): the Pipeline tab may clean the user's own items in `/private/tmp` and copy to the backup drive (`/Volumes/...`), see below
- Touching browser user-data dirs (caches only)
- Login Items / Launch Agents management
- Auto-clean scheduling: still out. The Pipeline watcher only notifies; every clean is one click by Rik
- Telemetry, auth, paywall, plugin ecosystem — all post-launch

## Source of truth
The cache catalog in `src-tauri/src/catalog.rs` IS the product — 42 categories as of v0.1 (expanded from the original 15 using path knowledge from the MIT-licensed Mole project). Adding a category = one struct literal. Keep entries grouped and tiered; never invent unsafe paths.

## Safety contract
1. Tier 1 paths are hard-coded to known cache dirs. No user-input globs in Tier 1.
2. Tier 2/3 deletions go through the system Trash via the `trash` crate, NOT direct `rm -rf`.
3. Refuse to clean a category if its owning app is running (see catalog `running_check`).
4. Every cleanup writes a record to `history.json` BEFORE returning.
5. `node_modules` scan is constrained to user-declared roots — never `~/` recursively.

## Today's ship bar
See §11 of prd.md (P0 acceptance checklist). If a change doesn't help check one of those boxes, defer it to P1.

## Style
- High contrast, big numbers, calm UI. Match `~/Documents/rik-docs/04_content-pipeline/drafts/when-claude-got-weak.html`.
- No emojis in UI.
- Tabular numerals everywhere a size or percentage appears.
- Light / dark mode auto-follow system.

## Build & run
```bash
pnpm install
pnpm tauri dev           # local
pnpm tauri build         # universal release
```

## Pipeline tab (24 Oct 2026)
Workflow-aware leftovers for Rik's video pipeline (RikGPT shorts, motion renders, render temp, Clip Desk boards, Codex logs). Code: `src-tauri/src/pipeline.rs`, UI `src/components/PipelineView.tsx`. Rules are data in `~/Library/Application Support/disk-doctor/pipeline.json` (seeded from `Config::default()` on first run; a new folder = one JSON rule, no code).
- Gate: an item is offered only when its short is finished in a run-state file (`done_stages`) or nothing in it changed for `min_age_days`.
- `trash` rules move files to the Trash. `offload` rules copy to the backup drive (`backup.root`/`offload_dir`/<rule>/<item>/), verify sha256, then trash the local file; a byte-identical copy already anywhere on the drive is reused (no copy). Nothing is removed before verification.
- Watcher (lib.rs): tray title shows free GB (`!` under the threshold); under the threshold it checks the rules every 10 min and sends at most one notification per `notify_every_hours`. It never cleans on its own.
- Check rules without the app: `cd src-tauri && cargo run --example pipeline_dry`. Tests: `cargo test --lib pipeline`.
- Release builds keep build-time code unstripped (`[profile.release.build-override] strip = false`): `strip = true` corrupts proc-macro dylibs on macOS 27.

## Tauri command surface (locked at 7)
```rust
disk_status() -> DiskStatus
memory_pressure() -> MemPressure
scan_all() -> Vec<CategoryResult>
scan_one(id: String) -> CategoryResult
clean(ids: Vec<String>, dry_run: bool) -> CleanResult
history() -> Vec<CleanRecord>
open_in_finder(path: String) -> ()
```

Do not add commands without updating the PRD.

Added since v0.1 (Memory tab, then Pipeline tab on 24 Oct 2026): `top_processes`, `open_activity_monitor`, `quit_process`, `pipeline_scan`, `pipeline_run(ids, dry_run)` (emits `pipeline-progress`), `pipeline_set_threshold(gb)`, `pipeline_edit_rules`, `empty_trash`.

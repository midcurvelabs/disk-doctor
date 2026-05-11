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
- Cleaning anything outside `~` (no `/Library`, no `/private/var`, no system caches)
- Touching browser user-data dirs (caches only)
- Login Items / Launch Agents management
- Auto-clean scheduling — P2
- Telemetry, auth, paywall, plugin ecosystem — all post-launch

## Source of truth
The 15-entry cache catalog in §7 of `prd.md` IS the product. Adding a category = one entry in `src-tauri/src/catalog.rs`. Do not invent new categories without confirming with Rik.

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

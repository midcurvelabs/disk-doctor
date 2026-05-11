# Disk Doctor

> CleanMyMac for the vibe coding era.

A macOS menubar app that audits and cleans the caches generic Mac cleaners miss — Claude Code agent VMs, opencode session snapshots, `node_modules` across many projects, Ollama models, Electron app partitions, Xcode artifacts, and the usual suspects.

Built in a day as Ship Season 01, Day 6. See the field-notes presentation: [when-claude-got-weak.html](https://github.com/midcurvelabs/disk-doctor#story).

## What it knows about

| Category | Tier | Notes |
|---|---|---|
| npm cache | 1 | `~/.npm/_cacache`, `~/.npm/_npx` |
| `node_modules` (all projects) | 1 | regenerates via `npm install` |
| opencode session snapshots | 1 | `~/.local/share/opencode/snapshot` |
| Codex archived sessions + logs DB | 1 | |
| Claude Code agent VM disk image | 2 | rebuilds on next agent run |
| Ollama models | 2 | lists them first, then removes |
| Xcode DerivedData + Archives | 1 | |
| iOS Simulators | 2 | |
| Descript electron partitions | 2 | refuses if Descript is open |
| Notion / Slack / Discord / Stremio caches | 2 | |
| Browser caches (Chrome / Brave / Dia) | 1 | refuses if browser is open |
| pip cache, Playwright binaries, Homebrew | 1 | |
| User logs | 1 | |
| Trash | 1 | |

## Install

Download the latest `.dmg` from [Releases](../../releases).

Or build from source:

```bash
git clone https://github.com/midcurvelabs/disk-doctor
cd disk-doctor
pnpm install
pnpm tauri dev      # local dev
pnpm tauri build    # universal release binary in src-tauri/target/release/bundle/
```

Requires Node 20+, Rust 1.78+, pnpm, macOS 13+.

## How it works

- **Read-only by default.** Disk Doctor lists what it finds; nothing leaves your machine without your click.
- **Tiered cleanup.** Tier 1 are pure caches that regenerate. Tier 2 are larger artifacts that rebuild on next use. Tier 3 are user-judgment calls.
- **Refuses to clean while apps are open.** Trying to wipe Descript's partitions while Descript runs? Refused.
- **History on disk.** Every cleanup writes a line to `~/Library/Application Support/disk-doctor/history.json`.

## NON-Goals

- Windows / Linux.
- Anything outside `$HOME` — no `/Library`, no `/private/var`, no system caches.
- Browser user-data dirs (history, bookmarks, logins) — caches only.
- Login Items / Launch Agent management.

## License

MIT. See [LICENSE](./LICENSE).

## Story

Built in a day after a 73% full disk plus 90 MCP servers crashed Claude Code mid-task. The diagnosis and cleanup story is in the Ship Season presentation. The app is the lesson productised.

— Rik · 2026

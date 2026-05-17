# Disk Doctor

> CleanMyMac for the vibe coding era.

A macOS menubar app that audits and cleans the caches generic Mac cleaners miss — Claude Code agent VMs, opencode session snapshots, `node_modules` across many projects, Ollama models, Electron app partitions, Xcode artifacts, and the usual suspects.

Built in a day as Ship Season 01, Day 6. See the field-notes presentation: [when-claude-got-weak.html](https://github.com/midcurvelabs/disk-doctor#story).

## What it knows about

**40+ cache categories**, grouped into three cleanup tiers. Highlights:

| Group | Examples |
|---|---|
| **Package managers** | npm, pnpm store, Yarn, Bun, pip, uv, Poetry, Conda, Cargo, Go, Gradle, Maven, Composer, NuGet, sbt |
| **AI / agent tools** | Claude Code agent VM, opencode snapshots, Codex sessions, Ollama models, Hugging Face / PyTorch / TensorFlow / W&B model caches, Copilot |
| **Build tooling** | Vite, webpack, Turborepo, Parcel, ESLint, Prettier, tsc, `node_modules` + build artifacts (`.next`, `dist`, `target`, `.venv`) across every project |
| **Xcode & Apple** | DerivedData, Archives, device support, iOS Simulators, QuickLook thumbnails |
| **Browsers** | Chrome, Brave, Dia, Edge, Firefox, Safari, Arc, Chromium, Comet caches |
| **Desktop apps** | Descript, Notion, Slack, Discord, Stremio, VS Code, Cursor, Zed, JetBrains, Adobe, Figma, Sketch, DaVinci, Postman, Insomnia, TablePlus |
| **DevOps & containers** | Docker buildx, Terraform, Bazel, SonarLint, Playwright / Puppeteer browser binaries, Homebrew downloads |
| **System** | user logs, crash/diagnostic reports, incomplete downloads, saved application state, Trash |

The full catalog lives in [`src-tauri/src/catalog.rs`](src-tauri/src/catalog.rs) — one struct literal per category. Adding a new one is a one-line PR.

Catalog path knowledge informed by the [Mole](https://github.com/tw93/Mole) project (MIT).

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

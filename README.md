# ContextLayer

![CI](https://github.com/sstanley-yelnatss/ContextLayer/actions/workflows/ci.yml/badge.svg)

**Local reasoning timelines for serious questions.**

ContextLayer is a desktop app for structured investigation. You open a **workspace** for a question you are working through (a product bet, a strategic call, a security assessment, a debugging rabbit hole). Each **block** on the timeline records what you believe, what you tried, what you observed, and what you concluded.

A **health panel** flags open loops, stale threads, and dead ends so unfinished reasoning does not disappear into scattered notes.

Data stays on your machine in SQLite (`%USERPROFILE%\.contextlayer\graph.db`).

## Install (Windows — most users)

**You do not need to clone the repo or install Rust.**

1. Go to **[Releases](https://github.com/sstanley-yelnatss/ContextLayer/releases)** on GitHub.
2. Download the latest **`ContextLayer_*_x64-setup.exe`**.
3. Run the installer. SmartScreen may warn about an unsigned build — **More info → Run anyway**.
4. Open **ContextLayer** from the Start menu.

The installer puts these in the same folder (e.g. `C:\Program Files\ContextLayer\`):

- `ContextLayer.exe` — desktop app
- `contextlayer-recorder.exe`, `contextlayer-mcp.exe`, `contextlayer-trace.exe` — bundled tools (no separate download)

**MCP in Cursor:** open the app → **Help** → **Copy MCP config** → paste into Cursor Settings → MCP.

**Clone / build from source** is only for contributors — see [Development](#development) below.

### MCP (recommended if you use Cursor or Claude)

ContextLayer ships with an optional **MCP server** that reads and writes the **same database** as the desktop app. While you investigate in chat, you can ask the agent to **log blocks**, **update evidence on a block by title**, **list what is in a workspace**, or **check hygiene** (orphans, stale items, dead ends). Less copy-paste from chat into the app; your reasoning graph stays current as you work.

Setup: [docs/MCP-SETUP.md](./docs/MCP-SETUP.md) · **Commands:** [docs/COMMANDS-CHEATSHEET.md](./docs/COMMANDS-CHEATSHEET.md) · MCP tools: [docs/MCP-TOOLS.md](./docs/MCP-TOOLS.md) · Agent prompts: [docs/mcp-cursor-cheatsheet.md](./docs/mcp-cursor-cheatsheet.md)

> **Not a notes app.** Typed hypothesis / action / evidence / conclusion fields, not a freeform vault. Cloud sync is not in this release.

### Documentation (start here)

| Doc | Use when |
|-----|----------|
| **[COMMANDS-CHEATSHEET.md](./docs/COMMANDS-CHEATSHEET.md)** | Quick reference — desktop, recorder CLI, capture gate, MCP essentials |
| [MCP-SETUP.md](./docs/MCP-SETUP.md) | Wire MCP into Cursor / Claude Desktop |
| [MCP-TOOLS.md](./docs/MCP-TOOLS.md) | Full MCP tool list + params |
| [mcp-cursor-cheatsheet.md](./docs/mcp-cursor-cheatsheet.md) | Example prompts + `save_block` fields |
| [TROUBLESHOOTING.md](./docs/TROUBLESHOOTING.md) | Something broke |

- [ContextLayerPRD.md](./ContextLayerPRD.md) — locked v1.0 spec
- [docs/PRD-addendum-blocks.md](./docs/PRD-addendum-blocks.md) — blocks, belief states, hygiene roadmap
- [docs/FUTURE-IMPLEMENTATION.md](./docs/FUTURE-IMPLEMENTATION.md) — post-beta backlog (PR export, collaboration, Graphify/Linear inspo)
- [docs/B1-PR-EXPORT-SPEC.md](./docs/B1-PR-EXPORT-SPEC.md) — Phase 1 PR export (multi-select → markdown)
- [docs/PITCH.md](./docs/PITCH.md) — concise read-off pitch
- [docs/DESIGN-PARTNER-OUTREACH.md](./docs/DESIGN-PARTNER-OUTREACH.md) — 30-day design partner ops (Twitter, DMs, metrics)
- [docs/PRD-addendum-merged-vision.md](./docs/PRD-addendum-merged-vision.md) — merged product scope (ContextLayer × GitLLM)

<img width="1917" height="982" alt="image" src="https://github.com/user-attachments/assets/d621b131-cf40-4d4c-abf3-068d3574283e" />

<img width="1362" height="677" alt="image" src="https://github.com/user-attachments/assets/cfea915e-15bf-42fb-913a-75b4884a9d8a" />

<img width="1917" height="982" alt="image" src="https://github.com/user-attachments/assets/08915599-81b3-43f7-94bf-261478a3495b" />





## Development

**For building from source only** — end users should use [Install (Windows)](#install-windows--most-users) above.

### Prerequisites

1. **Rust** — [rustup.rs](https://rustup.rs/) (required for Tauri)
2. **Node.js 20+**
3. **Tauri prerequisites (Windows)** — [tauri.app/start/prerequisites](https://tauri.app/start/prerequisites/)

### Windows: `cargo` not found after install

If `npm run tauri dev` says `program not found` for `cargo`, Rust is installed but your terminal session does not have it on `PATH` yet. Either **close and reopen the terminal** (or restart Cursor), or run once in that session:

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;" + $env:Path
```

### Build fails with `E0119` / `cookie` / `tauri-utils` (Jun 2026)

Rust 1.89+ plus `time` 0.3.48 triggers blanket-impl conflicts in Tauri’s dependency tree. This repo pins `time` to **0.3.47** in `Cargo.lock`. If you regenerate the lockfile, run:

```powershell
cargo update -p time --precise 0.3.47
```

Track upstream: [time-rs/time#783](https://github.com/time-rs/time/issues/783), [tauri-apps/tauri#15525](https://github.com/tauri-apps/tauri/issues/15525).

### Clone and run locally

Clone from GitHub (**Code** → copy URL), then:

```powershell
cd ContextLayer
npm run desktop:install   # once
npm run dev
```

Use the **Tauri desktop window**, not the Vite browser tab.

Alternative from `apps/desktop`:

```powershell
cd apps/desktop
npm install
npm run tauri dev
```

Database path: `%USERPROFILE%\.contextlayer\graph.db`

### MCP build

The Windows installer bundles `contextlayer-mcp.exe` next to the desktop app. After install, use **Help → Copy MCP config** in the app, or point Cursor at:

`%ProgramFiles%\ContextLayer\contextlayer-mcp.exe` (path varies by install location)

Dev build from repo root:

```powershell
cargo build -p contextlayer-mcp --release
npm run desktop:sidecars   # stage all CLI sidecars for Tauri bundle
```

Copy [`.cursor/mcp.json.example`](./.cursor/mcp.json.example) → `.cursor/mcp.json` and set the absolute path to `contextlayer-mcp.exe`.

The desktop app does not need to be running while MCP is in use.

## Workspace layout

```
ContextLayer/
├── apps/
│   ├── desktop/        # Tauri 2 + React UI
│   ├── mcp-server/     # stdio MCP read/write lane
│   ├── recorder/       # opt-in Cursor transcript capture
│   └── trace-cli/      # trace CI for PRs
├── crates/
│   ├── core/           # domain types + admission validation
│   ├── db/             # SQLite migrations
│   ├── export/         # compile + import
│   └── trace/          # capture log, recorder, trace CI
├── migrations/
└── fixtures/workspaces/
```

## What is (and is not) in this repo

**Shipped with the product:** source, migrations, `docs/`, [ContextLayerPRD.md](./ContextLayerPRD.md), and [`.cursor/mcp.json.example`](./.cursor/mcp.json.example) for MCP setup.

**Local only (gitignored):** `.taskmaster/` (internal task planning), `.cursor/mcp.json` and other Cursor IDE files, `.env`, `target/`, `node_modules/`, and `*.db` under your user profile. See [`.env.example`](./.env.example) for optional MCP env vars.

## Locked invariants (do not drift)

- Four stored types only — **Constraint is not first-class**
- Conclusions require ≥1 hypothesis + ≥1 evidence link
- Phase 1 UI: blocks are manual or MCP-logged; **opt-in** live capture via `start_capture` + recorder (not always-on)
- Compile outputs are views over the graph, never shadow schema

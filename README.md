# ContextLayer

**Local reasoning timelines for serious questions.**

ContextLayer is a desktop app for structured investigation. You open a **workspace** for a question you are working through (a product bet, a strategic call, a security assessment, a debugging rabbit hole). Each **block** on the timeline records what you believe, what you tried, what you observed, and what you concluded.

A **health panel** flags open loops, stale threads, and dead ends so unfinished reasoning does not disappear into scattered notes.

Data stays on your machine in SQLite (`%USERPROFILE%\.contextlayer\graph.db`).

### MCP (recommended if you use Cursor or Claude)

ContextLayer ships with an optional **MCP server** that reads and writes the **same database** as the desktop app. While you investigate in chat, you can ask the agent to **log blocks**, **update evidence on a block by title**, **list what is in a workspace**, or **check hygiene** (orphans, stale items, dead ends). Less copy-paste from chat into the app; your reasoning graph stays current as you work.

Setup: [docs/MCP-SETUP.md](./docs/MCP-SETUP.md) · Tool list & prompts: [docs/mcp-cursor-cheatsheet.md](./docs/mcp-cursor-cheatsheet.md)

> **Not a notes app.** Typed hypothesis / action / evidence / conclusion fields, not a freeform vault. Cloud sync is not in this release.

**Friends beta (source only):** build from source; see [docs/BETA-LAUNCH-CHECKLIST.md](./docs/BETA-LAUNCH-CHECKLIST.md), [docs/TROUBLESHOOTING.md](./docs/TROUBLESHOOTING.md), and [CONTRIBUTING.md](./CONTRIBUTING.md).

- [ContextLayerPRD.md](./ContextLayerPRD.md) — locked v1.0 spec
- [docs/PRD-addendum-blocks.md](./docs/PRD-addendum-blocks.md) — blocks, belief states, hygiene roadmap

## Prerequisites

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

## Development

```powershell
# Option A — from repo root (after first-time install below)
npm run desktop:install   # once
npm run dev

# Option B — from apps/desktop
cd apps/desktop
npm install
npm run tauri dev
```

Database path: `%USERPROFILE%\.contextlayer\graph.db`

### MCP (optional)

Build `contextlayer-mcp`, point your AI tool’s MCP config at it (**stdio**). Uses the **same database** as the app.

- **Setup (Cursor + Claude + others):** [docs/MCP-SETUP.md](./docs/MCP-SETUP.md)
- **Tools and prompts:** [docs/mcp-cursor-cheatsheet.md](./docs/mcp-cursor-cheatsheet.md)
- **Cursor:** copy [`.cursor/mcp.json.example`](./.cursor/mcp.json.example) → `.cursor/mcp.json` (absolute path to the binary)
- **Claude Desktop:** merge into `claude_desktop_config.json` (see MCP-SETUP)

Requires **Rust** to build MCP from source unless a prebuilt binary is attached to GitHub Releases.

```powershell
cargo build -p contextlayer-mcp --release
```

The desktop app does not need to be running while MCP is in use.

## Workspace layout

```
ContextLayer/
├── apps/
│   ├── desktop/        # Tauri 2 + React UI
│   └── mcp-server/     # stdio MCP read/write lane (dogfood)
├── crates/
│   ├── core/           # domain types + admission validation
│   ├── db/             # SQLite migrations
│   └── export/         # workspace summary compile prototype
├── migrations/
└── fixtures/workspaces/
```

## What is (and is not) in this repo

**Shipped with the product:** source, migrations, `docs/`, [ContextLayerPRD.md](./ContextLayerPRD.md), and [`.cursor/mcp.json.example`](./.cursor/mcp.json.example) for MCP setup.

**Local only (gitignored):** `.taskmaster/` (internal task planning), `.cursor/mcp.json` and other Cursor IDE files, `.env`, `target/`, `node_modules/`, and `*.db` under your user profile. Cloning the repo does not require Taskmaster or Cursor rules.

## Locked invariants (do not drift)

- Four stored types only — **Constraint is not first-class**
- Conclusions require ≥1 hypothesis + ≥1 evidence link
- Phase 1 UI: no auto-ingest; MCP is optional manual logging (your exact wording)
- Compile outputs are views over the graph, never shadow schema

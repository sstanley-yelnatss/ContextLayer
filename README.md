# ContextLayer

**Reasoning Graph System** — bounded epistemic tracking for investigative work.

> Workspaces → Hypotheses → Actions → Evidence → Conclusions.  
> Not a notes app. Not a compile platform. Cross-AI continuity is Phase 4.

- [ContextLayerPRD.md](./ContextLayerPRD.md) — locked v1.0 spec
- Taskmaster: `.taskmaster/tasks/tasks.json`

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
# From repo root (after Rust is installed)
cd apps/desktop
npm install
npm run tauri dev
```

Database path: `%USERPROFILE%\.contextlayer\graph.db`

### Cursor MCP (dogfood lane)

Thin read/write bridge to the same SQLite file as the desktop app. Text is stored **verbatim** — no normalization.

```powershell
# Build once (from repo root)
cargo build -p contextlayer-mcp
```

Project config: `.cursor/mcp.json` points at `target/debug/contextlayer-mcp.exe`. **Restart MCP servers** in Cursor after building (Settings → MCP → refresh, or reload window).

| Tool | Use when |
|------|----------|
| `list_workspaces` | You need a workspace id |
| `get_workspace_summary` | Before suggesting tests — avoid retesting ruled-out paths |
| `create_workspace` | New bounty program / CTF |
| `create_hypothesis` | Log a claim in your own words |
| `create_action` | Log what you did (curl, scan, manual step) |
| `create_evidence` | Log raw output |
| `add_link` | Wire hypothesis→action→evidence |
| `save_conclusion` | Interpretation (needs hypothesis + evidence links) |

Example prompts in chat:

- *"Log this as a hypothesis in my Acme workspace: gonna test IDOR at /api/users/{id}"*
- *"Create action + evidence for the last curl, link them, then show workspace summary"*
- *"Save conclusion: IDOR not possible here — rejected — link to hypothesis X and evidence Y"*

Override DB path: set env `CONTEXTLAYER_DB` on the MCP server process.

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

## Taskmaster

```powershell
task-master list
task-master next
```

## Locked invariants (do not drift)

- Four stored types only — **Constraint is not first-class**
- Conclusions require ≥1 hypothesis + ≥1 evidence link
- Phase 1 UI: no auto-ingest; MCP is optional manual logging (your exact wording)
- Compile outputs are views over the graph, never shadow schema

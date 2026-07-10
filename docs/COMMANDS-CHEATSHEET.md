# ContextLayer — command cheat sheet

One-page reference for **desktop**, **CLI binaries**, and **MCP**. Same data everywhere: `%USERPROFILE%\.contextlayer\graph.db` (blocks) + `%USERPROFILE%\.contextlayer\capture\` (session logs).

> **Doc map:** Setup → [MCP-SETUP.md](./MCP-SETUP.md) · MCP tools (full) → [MCP-TOOLS.md](./MCP-TOOLS.md) · Agent prompts → [mcp-cursor-cheatsheet.md](./mcp-cursor-cheatsheet.md) · This file = **commands only**.

---

## Build (from repo root)

```powershell
cd path\to\ContextLayer

npm run desktop:install   # once
npm run dev               # Tauri window — not the browser tab

# Windows installer (desktop + bundled recorder, MCP, trace CLI)
npm run desktop:build
```

`desktop:build` runs `scripts/prepare-sidecars.ps1` (builds CLI binaries and stages them for Tauri) **then** `tauri build`. You do not run those as separate manual steps.

Dev-only (without full installer):

```powershell
npm run desktop:sidecars   # stage sidecars for local Tauri dev bundle
cargo build -p contextlayer-mcp -p contextlayer-recorder -p contextlayer-trace-cli --release
```

Binaries: `target\release\contextlayer-mcp.exe`, `contextlayer-recorder.exe`, `contextlayer-trace.exe` — also bundled beside `ContextLayer.exe` in the Windows installer.

---

## Desktop (no CLI required)

| Do this | Where |
|--------|--------|
| Create / open workspace | Home → **New workspace** or click a title |
| Log reasoning | Timeline → add/edit blocks (H / A / E / C) |
| Check health | Hygiene panel (orphans, stale, dead ends) |
| Export for PR | Select blocks → **Export for PR** |
| **Start / stop capture** | Timeline toolbar — opens gate **and** ingests Cursor chat automatically (no terminal) |
| Copy agent packet | MCP `compile_agent_context` (desktop button removed — use MCP/skill) |

Workspace **name** is what you set at create time — use that string in CLI/MCP below (UUID not shown in UI).

---

## Live capture — opt-in only

Nothing records until you **start a session**. Binding a repo is just a map, not record.

### Flow (desktop — recommended)

```
1. Open workspace in ContextLayer
2. Start capture in toolbar     (auto-binds git repo when possible; ingest starts immediately)
3. Work in Cursor + log blocks  (MCP or desktop)
4. Checkpoint / branch tangents (MCP: branch_capture_session — optional)
5. Stop capture when done
```

### Flow (CLI / scripts)

```
1. bind repo → workspace     (once per project — desktop Start capture tries this automatically)
2. start capture session
3. recorder watch            (only if desktop app is not running ingest for you)
4. log blocks + checkpoints
5. stop capture session
```

### When to use the CLI (edge cases)

| Situation | Use app Start capture | Use recorder CLI |
|-----------|----------------------|------------------|
| Normal demo / daily work | Yes — gate + ingest | Not needed |
| Logging blocks via MCP, app open | Yes | Not needed |
| MCP `start_capture`, **desktop closed** | Open app, or… | Run `watch` in a terminal |
| Bind non-git folder or `--cursor-project` | Auto-bind may fail | `bind-repo` / `start --cursor-project …` |
| Import a saved transcript file | — | `import --file …` |
| Debug “why is ingest empty?” | Check capture on + binding | `status`, `once`, `list-bindings` |
| Script/automation | — | `start` / `stop` / `watch` |

**Default:** installer users never touch the CLI. It ships so advanced flows and MCP-without-GUI still work.

### Recorder CLI (`contextlayer-recorder`)

Use workspace **title** from desktop (quotes if spaces):

```powershell
# See names ↔ ids
contextlayer-recorder list-workspaces

# Map Cursor project → workspace (no recording yet)
contextlayer-recorder bind-repo `
  --path C:\path\to\repo `
  --workspace "ContextLayer product validation"

# Start / stop the gate (only NEW chat lines after start are ingested)
contextlayer-recorder start --workspace "ContextLayer product validation"
contextlayer-recorder start --workspace "My Hunt" --cursor-project c-Users-miles-MyHunt
contextlayer-recorder stop --workspace "ContextLayer product validation"

# Background poll (no-op when no active session)
contextlayer-recorder watch
contextlayer-recorder once          # single poll, print stats

# Onboarding — explicit import, not gated
contextlayer-recorder import --workspace "My Hunt" --file C:\path\to\transcript.jsonl

contextlayer-recorder status        # active capture sessions
contextlayer-recorder list-bindings
```

| Flag | Purpose |
|------|---------|
| `--workspace` | UUID **or** exact workspace name (case-insensitive) |
| `--cursor-project` | Only ingest transcripts from this Cursor project folder key |
| `--transcript` | Only one chat file (current convo) |
| `--label` | Optional note on the session |

### MCP capture (same gate — ingest needs app or `watch`)

| Tool | When |
|------|------|
| `bind_capture_project` | Map Cursor project → workspace (no recording) |
| **`start_capture`** | Open gate only — **does not poll**. Pair with desktop Start capture or CLI `watch` |
| **`stop_capture`** | Close gate |
| **`capture_status`** | List active sessions |
| `get_context_log` | Read session log window |
| `get_context_commits` | Read decision commits |
| `commit_checkpoint` | Decision moment — slices log seq range (not every prompt) |
| `branch_capture_session` / `merge_capture_branch` | Fork a tangent chat thread within capture (requires active capture) |
| `import_session` | Paste transcript → new workspace + draft blocks + log backfill |

Example chat: *"Start ContextLayer capture for workspace ContextLayer product validation, then log a hypothesis block …"*

---

## MCP — everyday tools

**Read (tiered — don’t dump everything):**

| Tool | Tier |
|------|------|
| `list_workspaces` | 0 — pick workspace |
| `get_workspace_index` | 1 — titles + belief + flags, **no bodies** |
| `get_block` | 2 — one full block |
| `get_workspace_hygiene` | Before suggesting next tests |

**Write:**

| Tool | Notes |
|------|--------|
| **`save_block`** | Primary. Partial updates: only send changed fields. Target by `block_title`. |
| `create_workspace` | New workspace |

**Export:**

| Tool | Notes |
|------|--------|
| `export_blocks` | PR markdown (`block_ids` / `block_titles`) |
| `compile_agent_context` | Full agent handoff packet |

Full list + params: [MCP-TOOLS.md](./MCP-TOOLS.md)

---

## Trace CI (PRs)

```powershell
cargo run -p contextlayer-trace-cli -- check `
  --rules .contextlayer/rules.yml `
  --pr-body-text "..." `
  --repo .
```

Rules file: `.contextlayer/rules.yml` · GitHub Action: `.github/workflows/trace-ci.yml`

---

## Where files live

| Path | What |
|------|------|
| `~/.contextlayer/graph.db` | Blocks, workspaces, hygiene |
| `~/.contextlayer/capture/{workspace_id}/log.jsonl` | Session message stream |
| `~/.contextlayer/capture/{workspace_id}/commits.jsonl` | Decision checkpoints |
| `~/.contextlayer/capture_sessions.json` | Active opt-in capture sessions |
| `~/.contextlayer/bindings.json` | Cursor project → workspace map |
| `~/.contextlayer/recorder_state.json` | Transcript tail offsets |

---

## Quick troubleshooting

| Problem | Fix |
|---------|-----|
| MCP tools missing / stale | Disable MCP → rebuild `contextlayer-mcp` → re-enable Cursor MCP |
| Recorder ingests nothing | Capture off? Start capture in app or `start`. App open + capture on but empty? Confirm git repo binding (`bind-repo` once if auto-bind missed). |
| Wrong workspace | `list-workspaces` / `list_workspaces` — use exact title |
| Duplicate workspace names | Rename in desktop or use UUID |
| Desktop vs MCP out of sync | Same `graph.db`; **capture ingest needs desktop open** (or CLI `watch`) while session is active |

More: [TROUBLESHOOTING.md](./TROUBLESHOOTING.md)

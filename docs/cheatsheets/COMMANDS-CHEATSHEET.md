# ContextLayer — command cheat sheet

One-page reference for **desktop**, **CLI binaries**, and **MCP**. Same data everywhere: `%USERPROFILE%\.contextlayer\graph.db` (blocks) + `%USERPROFILE%\.contextlayer\capture\` (session logs).

> **Doc map:** Setup → [MCP-SETUP.md](./MCP-SETUP.md) · MCP tools (full) → [MCP-TOOLS.md](./MCP-TOOLS.md) · Agent prompts → [mcp-cursor-cheatsheet.md](./mcp-cursor-cheatsheet.md) · This file = **commands only**.

---

## Build (from repo root)

```powershell
cd path\to\ContextLayer

# Desktop
npm run desktop:install   # once
npm run dev               # Tauri window — not the browser tab

# MCP + recorder + trace CI
cargo build -p contextlayer-mcp -p contextlayer-recorder -p contextlayer-trace-cli --release
```

Binaries: `target\release\contextlayer-mcp.exe`, `contextlayer-recorder.exe`, `contextlayer-trace.exe`

---

## Desktop (no CLI)

| Do this | Where |
|--------|--------|
| Create / open workspace | Home → **New workspace** or click a title |
| Log reasoning | Timeline → add/edit blocks (H / A / E / C) |
| Check health | Hygiene panel (orphans, stale, dead ends) |
| Export for PR | Select blocks → **Export for PR** |
| Copy agent packet | MCP `compile_agent_context` (desktop button removed — use MCP/skill) |

Workspace **name** is what you set at create time — use that string in CLI/MCP below (UUID not shown in UI).

---

## Live capture — opt-in only

Nothing records until you **start a session** and pick a chat thread. Binding a repo is just a map, not record.

**Transcript sources:** Cursor agent chats and **Claude Code** session JSONL (`~/.claude/projects/`). Not the consumer Claude Desktop chat app.

### Flow

```
1. bind repo → workspace     (once per project)
2. start capture session     (when investigation begins)
3. recorder watch            (background poll — optional but typical)
4. log blocks + checkpoints  (MCP or desktop)
5. stop capture session      (when done)
```

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

### MCP capture (same gate)

| Tool | When |
|------|------|
| `bind_capture_project` | Map Cursor project → workspace (no recording) |
| **`start_capture`** | Open gate — `workspace` = name or UUID; optional `cursor_project`, `transcript_path` |
| **`stop_capture`** | Close gate |
| **`capture_status`** | List active sessions |
| `get_context_log` | Read session log window |
| `get_context_commits` | Read decision commits |
| `commit_checkpoint` | Decision moment — slices log seq range (not every prompt) |
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
| Recorder ingests nothing | Run `capture_status` or `recorder status` — need `start` first |
| Wrong workspace | `list-workspaces` / `list_workspaces` — use exact title |
| Duplicate workspace names | Rename in desktop or use UUID |
| Desktop vs MCP out of sync | Same `graph.db`; desktop need not be open for MCP |

More: [TROUBLESHOOTING.md](./TROUBLESHOOTING.md)

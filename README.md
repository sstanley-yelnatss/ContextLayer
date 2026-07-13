# ContextLayer

![CI](https://github.com/sstanley-yelnatss/ContextLayer/actions/workflows/ci.yml/badge.svg)

**Local reasoning timelines for serious questions.**

ContextLayer is a desktop app for structured investigation. You open a **workspace** for a question you are working through (a product bet, a strategic call, a security assessment, a debugging rabbit hole). Each **block** on the timeline records what you believe, what you tried, what you observed, and what you concluded.

A **health panel** flags open loops, stale threads, and dead ends so unfinished reasoning does not disappear into scattered notes.

Data stays on your machine in SQLite (`%USERPROFILE%\.contextlayer\graph.db`).

## Install (Windows)

**You do not need to clone the repo or install Rust.**

1. Go to **[Releases](https://github.com/sstanley-yelnatss/ContextLayer/releases)** on GitHub.
2. Download the latest **`ContextLayer_*_x64-setup.exe`**.
3. Run the installer. SmartScreen may warn about an unsigned build. Choose **More info → Run anyway**.
4. Open **ContextLayer** from the Start menu.

The installer puts these in the same folder (e.g. `C:\Program Files\ContextLayer\`):

- `ContextLayer.exe`: desktop app
- `contextlayer-recorder.exe`, `contextlayer-mcp.exe`, `contextlayer-trace.exe`: bundled tools (no separate download)

**MCP in Cursor:** open the app → **Help** → **Copy MCP config** → paste into Cursor Settings → MCP.

## Using ContextLayer

| Task | Where |
|------|--------|
| Log reasoning blocks | Timeline in the desktop app |
| Check open loops / hygiene | Hygiene panel |
| Export for a PR | PR export mode → select blocks → copy |
| Live chat capture (optional) | **Start capture** in the toolbar |
| Agent logging from your editor | MCP. See docs below |

### Live capture (optional)

**Start capture** in the toolbar opens a picker of recent chat threads. New messages from the thread you pick are recorded into that workspace’s session log while the app is open (no separate terminal).

**Supported today**

| Source | What works |
|--------|------------|
| **Cursor** | Agent chat transcripts under `%USERPROFILE%\.cursor\projects\` |
| **Claude Code** | Session JSONL under `%USERPROFILE%\.claude\projects\` (CLI, VS Code / JetBrains integrations, and the [Claude Code](https://code.claude.com/docs/en/quickstart) desktop app) |

**Not supported:** the consumer **Claude Desktop** app (general claude.ai chat). That product does not write the session files ContextLayer reads. If capture does not show up after chatting, check for `.jsonl` files under `.claude\projects\`.

Capture ingests only **new** messages after you start the session, not the full prior history. If several chats were active recently, pick the right one in the picker. You can remember that choice per workspace.

> **Not a notes app.** Typed hypothesis / action / evidence / conclusion fields, not a freeform vault. Cloud sync is not in this release.

### Documentation

| Doc | Use when |
|-----|----------|
| **[COMMANDS-CHEATSHEET.md](./docs/cheatsheets/COMMANDS-CHEATSHEET.md)** | Desktop, capture, CLI edge cases |
| [MCP-SETUP.md](./docs/cheatsheets/MCP-SETUP.md) | Wire MCP into Cursor |
| [MCP-TOOLS.md](./docs/cheatsheets/MCP-TOOLS.md) | Full MCP tool list |
| [mcp-cursor-cheatsheet.md](./docs/cheatsheets/mcp-cursor-cheatsheet.md) | Example agent prompts |
| [TROUBLESHOOTING.md](./docs/cheatsheets/TROUBLESHOOTING.md) | Something broke |

In-app **Help** covers install paths, MCP config copy, and day-to-day capture.

<img width="1917" height="982" alt="image" src="https://github.com/user-attachments/assets/d621b131-cf40-4d4c-abf3-068d3574283e" />

<img width="1362" height="677" alt="image" src="https://github.com/user-attachments/assets/cfea915e-15bf-42fb-913a-75b4884a9d8a" />

<img width="1917" height="982" alt="image" src="https://github.com/user-attachments/assets/08915599-81b3-43f7-94bf-261478a3495b" />

## Development

**For contributors building from source.** End users should use [Install](#install-windows) above.

### Prerequisites

1. **Rust:** [rustup.rs](https://rustup.rs/)
2. **Node.js 20+**
3. **Tauri prerequisites (Windows):** [tauri.app/start/prerequisites](https://tauri.app/start/prerequisites/)

### Clone and run

```powershell
git clone https://github.com/sstanley-yelnatss/ContextLayer.git
cd ContextLayer
npm run desktop:install   # once
npm run dev
```

Use the **Tauri desktop window**, not the Vite browser tab.

### Build Windows installer

```powershell
npm run desktop:build
```

Output: `target\release\bundle\nsis\ContextLayer_*_x64-setup.exe`

### MCP (dev)

Installer users: **Help → Copy MCP config** in the app.

From source, copy [`.cursor/mcp.json.example`](./.cursor/mcp.json.example) → `.cursor/mcp.json` and point at `contextlayer-mcp.exe` (install dir or `target\release\` after `npm run desktop:sidecars`).

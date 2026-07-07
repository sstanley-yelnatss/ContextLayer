# ContextLayer — MCP setup (Cursor, Claude Desktop, others)

Optional. Log and read reasoning blocks from an AI tool that supports **local stdio MCP**.

**Not connected to the desktop app directly.** Both talk to the same SQLite file:

| Platform | Default database path |
|----------|------------------------|
| Windows | `%USERPROFILE%\.contextlayer\graph.db` |
| macOS / Linux | `~/.contextlayer/graph.db` |

Open the desktop app once if you want a UI on the same data (optional).

**After setup:** [COMMANDS-CHEATSHEET.md](./COMMANDS-CHEATSHEET.md) · full tool list: [MCP-TOOLS.md](./MCP-TOOLS.md)

---

## 1. Get the MCP binary

From a clone of this repo (requires [Rust](https://rustup.rs/)):

```powershell
# Windows — from repo root
cargo build -p contextlayer-mcp --release
```

Binary location:

| OS | Path |
|----|------|
| Windows | `target\release\contextlayer-mcp.exe` |
| macOS / Linux | `target/release/contextlayer-mcp` |

**Tip:** Copy the binary to a stable folder (e.g. `C:\Tools\contextlayer-mcp.exe` or `~/bin/contextlayer-mcp`) so config paths do not break when you delete `target/`.

**GitHub Releases (when available):** a prebuilt `contextlayer-mcp` may be attached so friends skip `cargo build`. Same config below; point `command` at the downloaded file.

Override database path (optional):

```json
"env": {
  "CONTEXTLAYER_DB": "C:\\Users\\YOU\\.contextlayer\\graph.db"
}
```

---

## 2. Cursor

### Project-level (recommended for this repo)

1. Copy [`.cursor/mcp.json.example`](../.cursor/mcp.json.example) to `.cursor/mcp.json`.
2. Set `command` to the **absolute path** to your MCP binary.
3. Cursor → **Settings → MCP** → refresh (or reload window).

**Example** (Windows):

```json
{
  "mcpServers": {
    "contextlayer": {
      "command": "C:\\Tools\\contextlayer-mcp.exe",
      "args": []
    }
  }
}
```

**Example** (macOS / Linux):

```json
{
  "mcpServers": {
    "contextlayer": {
      "command": "/Users/you/bin/contextlayer-mcp",
      "args": []
    }
  }
}
```

### Global Cursor MCP

Same JSON shape in Cursor **Settings → MCP** if you want ContextLayer in every project.

### Verify

In chat: *"List my ContextLayer workspaces"* → should call `list_workspaces`.

Tool reference: [mcp-cursor-cheatsheet.md](./mcp-cursor-cheatsheet.md).

---

## 3. Claude Desktop

Claude uses one config file for all MCP servers. **Merge** the `contextlayer` entry into your existing `mcpServers`; do not replace other servers.

### Config file location

| OS | Path |
|----|------|
| Windows | `%APPDATA%\Claude\claude_desktop_config.json` |
| macOS | `~/Library/Application Support/Claude/claude_desktop_config.json` |

Create the file if it does not exist.

### Example (Windows)

```json
{
  "mcpServers": {
    "contextlayer": {
      "command": "C:\\Tools\\contextlayer-mcp.exe",
      "args": []
    }
  }
}
```

If you already have other servers:

```json
{
  "mcpServers": {
    "some-other-server": {
      "command": "...",
      "args": []
    },
    "contextlayer": {
      "command": "C:\\Tools\\contextlayer-mcp.exe",
      "args": []
    }
  }
}
```

### Example (macOS)

```json
{
  "mcpServers": {
    "contextlayer": {
      "command": "/Users/you/bin/contextlayer-mcp",
      "args": []
    }
  }
}
```

Restart Claude Desktop completely after saving.

### Verify

Ask: *"Use ContextLayer to list my workspaces"* or *"Call list_workspaces on ContextLayer"*.

---

## 4. Other AI tools (Windsurf, VS Code extensions, etc.)

Any client that supports **stdio MCP** uses the same pattern:

- **command:** absolute path to `contextlayer-mcp` (`.exe` on Windows)
- **args:** `[]`
- **env (optional):** `CONTEXTLAYER_DB` if not using the default path

Check that tool’s docs for where to put `mcpServers` JSON.

**Does not work:** ChatGPT web (no local MCP), remote-only MCP hosts without running the binary locally.

---

## 5. What to tell the agent

ContextLayer tools are documented in [mcp-cursor-cheatsheet.md](./mcp-cursor-cheatsheet.md). Short version:

- **Read first:** `list_workspaces`, `list_blocks`, `get_workspace_summary`, `get_workspace_hygiene`
- **Write:** prefer **`save_block`** (partial updates: only send fields you change)
- Text is stored **verbatim**; only log when you ask

Example prompts:

- *"List blocks in my Acme workspace"*
- *"Add evidence to the IDOR block: HTTP 403 on /api/user/2"*
- *"What's ruled out in this workspace?"* (summary + hygiene)

---

## Troubleshooting

See [TROUBLESHOOTING.md](./TROUBLESHOOTING.md) (MCP exe locked, wrong DB, build errors).

Common fixes:

- **Path wrong:** use absolute path; escape backslashes in JSON on Windows (`\\`)
- **No workspaces yet:** create one in the desktop app or ask the agent to `create_workspace`
- **Changes not in app:** confirm MCP and app use the same `CONTEXTLAYER_DB` (or both default)

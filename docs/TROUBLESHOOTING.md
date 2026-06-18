# ContextLayer — Troubleshooting

## `ENOENT: no such file or directory, open '...\package.json'`

You ran `npm run tauri dev` from the **repo root**. Tauri lives in `apps/desktop`.

```powershell
cd apps/desktop
npm install
npm run tauri dev
```

Or from repo root (after `npm install` in `apps/desktop`):

```powershell
npm run dev
```

---

## `cargo` / `program not found`

Rust is installed but not on PATH in this terminal. Restart the terminal or:

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;" + $env:Path
```

---

## Build fails with `E0119` / `cookie` / `tauri-utils`

Rust 1.89+ and `time` 0.3.48 conflict with Tauri deps. This repo pins `time` 0.3.47. If you regenerated `Cargo.lock`:

```powershell
cargo update -p time --precise 0.3.47
```

---

## `There is not enough space on the disk` (os error 112)

Rust `target/` can grow to **5–10+ GB**.

```powershell
Remove-Item -Recurse -Force .\target -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force .\apps\desktop\src-tauri\target -ErrorAction SilentlyContinue
```

Ensure **≥5 GB** free, then rebuild.

---

## MCP build: `failed to remove contextlayer-mcp.exe` / Access denied

Cursor (or another process) has the MCP server running. Disable or refresh MCP in Cursor Settings, then:

```powershell
cargo build -p contextlayer-mcp
```

---

## App opens in browser instead of desktop window

Use the **Tauri** window that opens separately. Do not use the Vite dev URL as the product UI.

---

## MCP changes not showing in desktop app (or vice versa)

Both use the same DB by default: `%USERPROFILE%\.contextlayer\graph.db`.  
If you set `CONTEXTLAYER_DB` for MCP, the app must use the same path. Restart MCP after rebuilding the binary.

---

## Windows SmartScreen on installer

Unsigned builds show “Unknown publisher.” Expected for friends beta. Click through or build from source.

---

## Hypothesis rejected: “falsifiable claim”

Admission rules are heuristic, not AI. Common cause: hypothesis text **under 8 characters** after trim. Use a full sentence claim.

---

## More help

- [MCP-SETUP.md](./MCP-SETUP.md)
- [mcp-cursor-cheatsheet.md](./mcp-cursor-cheatsheet.md)
- [BETA-LAUNCH-CHECKLIST.md](./BETA-LAUNCH-CHECKLIST.md)
- Open a GitHub Issue with OS, command you ran, and full error text.

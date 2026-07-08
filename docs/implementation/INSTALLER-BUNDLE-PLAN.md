# Installer bundle plan — ship the whole product

**Goal:** One Windows installer ships desktop + recorder + MCP + trace CLI. Capture works from the toolbar without a separate terminal.

---

## Inventory (what must ship)

| Artifact | Role | Bundling |
|----------|------|----------|
| `ContextLayer.exe` | Desktop UI | Tauri main binary |
| `contextlayer-recorder.exe` | CLI capture / bind / import | Tauri `externalBin` sidecar |
| `contextlayer-mcp.exe` | Cursor MCP stdio server | Tauri `externalBin` sidecar |
| `contextlayer-trace.exe` | PR trace CI (`check`) | Tauri `externalBin` sidecar |
| `trace-rules.yml` | Sample rules (resource) | Tauri `bundle.resources` |

**Not separate binaries (already in app):** SQLite graph, capture store, export compile, in-app poll loop.

**User data (never in installer):** `%USERPROFILE%\.contextlayer\`

---

## Build pipeline

1. `scripts/prepare-sidecars.ps1` — `cargo build -p contextlayer-recorder -p contextlayer-mcp -p contextlayer-trace-cli --release`, copy to `apps/desktop/src-tauri/binaries/*-$TARGET_TRIPLE.exe`
2. `npm run desktop:build` — sidecars first, then `tauri build`
3. Installer output: all four `.exe` files in the same install directory

---

## Runtime UX

| Feature | Behavior |
|---------|----------|
| **Start capture** | Opens gate + starts in-app poll loop (same as `recorder watch`) + auto `bind-repo` from git root when detectable |
| **Stop capture** | Closes gate; stops poll loop when no active sessions |
| **App exit** | Stops poll loop |
| **Help → MCP** | Shows install paths + **Copy MCP config** with absolute path to bundled `contextlayer-mcp.exe` |
| **Recorder CLI** | Still available in install folder for scripts; not required for normal capture |

---

## MCP vs recorder

- **MCP** can `start_capture` / `stop_capture` (gate only). It does **not** poll transcripts.
- **Desktop** runs the poll loop while capture is active, so MCP + toolbar share the same ingest path when the app is open.

---

## Release checklist (Windows)

- [ ] `npm run desktop:build` from repo root on `main`
- [ ] Confirm install dir contains all four exes (or inspect NSIS staging)
- [ ] Smoke: Start capture → chat in Cursor → log grows → PR export with trace
- [ ] Help → Copy MCP config → paste in Cursor → `list_workspaces` works
- [ ] Tag + GitHub Release with setup `.exe` only (sidecars inside installer)

---

## Future (post demo)

- macOS / Linux sidecar triples + CI matrix
- Optional: “Install MCP to Cursor” helper (still user-confirmed; no silent patch)
- Code-sign installer (SmartScreen)

---

## Key files

| Area | Path |
|------|------|
| Sidecar script | `scripts/prepare-sidecars.ps1` |
| Tauri bundle | `apps/desktop/src-tauri/tauri.conf.json` |
| Capture poll | `apps/desktop/src-tauri/src/capture_watcher.rs` |
| Tool paths API | `get_bundled_tool_paths` in `lib.rs` |
| Help / MCP copy | `apps/desktop/src/pages/HelpPage.tsx` |

# ContextLayer — Friends beta checklist

**Status:** Repo is **public**. Source-only friends beta; pre-release on [GitHub Releases](https://github.com). **Clone `main`** for the latest README and fixes (release tags are snapshots).

---

## Quick start (for you and friends)

From a clone of **`main`**:

```powershell
npm run desktop:install   # once, from repo root
npm run dev
```

Or from `apps/desktop`: `npm install` then `npm run tauri dev`.

Use the **Tauri desktop window**, not the Vite browser tab.

**Optional MCP:** See [MCP-SETUP.md](./MCP-SETUP.md) (Cursor, Claude Desktop, others). Tool list: [mcp-cursor-cheatsheet.md](./mcp-cursor-cheatsheet.md).

```powershell
cargo build -p contextlayer-mcp --release
```

**Data:** `%USERPROFILE%\.contextlayer\graph.db` (shared by app + MCP).

---

## Must-do before public

- [x] **Add LICENSE** (MIT — see root `LICENSE`)
- [x] **Secrets scrub:** `.env`, `.cursor/mcp.json`, `*.db` not in tracked files (re-check after each push)
- [x] **Internal tooling out of git:** `.taskmaster/` and `.cursor/*` except `mcp.json.example` (see root `.gitignore`)
- [x] **Confirm `target/` / `node_modules` / `dist` gitignored** and not tracked
- [x] **Refresh README** for current MVP (product description, MCP callout, doc links)
- [ ] **Smoke test on a clean mental pass:** clone → install → dev → workspace → block → optional MCP
- [x] **GitHub repo public** — description, topics, Issues on (optional: Discussions)
- [x] **Tag a release:** e.g. `v0.1.0-mvp` (source-only; note “build from README / clone main”)

### Friends beta expectations (put in README)

Copy or adapt:

> **What works:** Desktop app, optional Cursor MCP, local SQLite reasoning graph (workspaces → blocks with hypothesis / action / evidence / conclusion), workspace health sidebar.  
> **What doesn’t yet:** Pre-built installers for everyone, Mac/Linux builds from maintainer, cloud sync, Obsidian export.  
> **Setup:** ~15–30 min (Rust + Node + [Tauri prerequisites](https://tauri.app/start/prerequisites/)). **Windows-first**; others build from source.  
> **Not a notes app:** Structured investigation reasoning, not a repo docs vault.

---

## Strongly recommended

- [ ] **One screenshot** in README (timeline + workspace health)
- [x] **CONTRIBUTING.md** (short: fork, issue for big changes, PRs welcome)
- [x] **TROUBLESHOOTING.md** linked from README
- [x] **Taskmaster / Cursor note** in README: internal folders gitignored; PRD lives at repo root + `docs/PRD-addendum-blocks.md`
- [ ] **Invite message** ready (pitch + 10-minute first session below)
- [ ] **One “first clone” friend** to run through setup and report friction before wider invite

### 10-minute first session (send to testers)

1. Create a workspace with a real **goal**
2. Add 2–3 **blocks** with **titles**
3. Fill hypothesis / action / evidence on at least one block
4. Open **Workspace health**; click Orphans or Stale
5. (Optional) Enable MCP; ask Cursor to `list_blocks` and add evidence to a block by title

---

## Packaging: `.exe` / installer worth it?

| Approach | Pros | Cons |
|----------|------|------|
| **Source-only beta (now)** | Fast to ship; friends who dev already have Rust/Node; easy fixes | High setup friction; easy to run wrong command |
| **Windows `.msi` / `.exe` (Tauri build)** | Friends double-click; no terminal for **app** | Still need separate MCP build for Cursor; ~5–10 GB `target/` during build; **SmartScreen** “unknown publisher”; no auto-update; Mac friends still stuck |

**Recommendation for this beta:**

1. **Ship source-first** with clear docs (this checklist + troubleshooting).
2. **Optional:** build a Windows installer for 2–3 non-dev friends only:
   ```powershell
   cd apps/desktop
   npm run tauri build
   ```
   Artifacts under `apps/desktop/src-tauri/target/release/bundle/`. Attach to GitHub Release with “Windows only, unsigned, click through SmartScreen.”
3. **Do not block public repo** on installers; add `.exe` when someone actually needs it.

MCP will always be a separate binary + `.cursor/mcp.json` until you bundle it inside the installer (later polish).

---

## Product scope reminders (don’t scope-creep before beta)

**In scope for MVP message:**

- Reasoning graph (H/A/E/C), belief states, workspace health, MCP `save_block` partial updates, block titles

**Out of scope for beta promise:**

- Obsidian vault generation, repo codebase map, ADR lifecycle, wikilinks/canvas in-app
- Cross-AI sync (Phase 4)
- Phase 3 **export-to-vault** is a good post-beta direction, not required for first invite

---

## Post-launch (collect feedback)

- [ ] Track: setup failures, disk space / `target/` issues, MCP confusion, UI polish
- [ ] GitHub Issues labels: `bug`, `question`, `idea`
- [ ] After feedback: README tweaks, optional Windows Release asset, hygiene/README refresh

---

## Maintainer pre-flight (you, today)

- [ ] Disk space: **≥5 GB free** before `tauri dev` / `cargo build` (delete `target/` if needed)
- [ ] Run from **`apps/desktop`** or `npm run dev` from root
- [ ] Last UI pass in Tauri window
- [ ] Push to GitHub → tag → send invite with link to this checklist

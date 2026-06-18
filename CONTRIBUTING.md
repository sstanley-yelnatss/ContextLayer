# Contributing to ContextLayer

Thanks for trying ContextLayer. This project is in **friends beta**: APIs and UX may change.

## Quick start

1. Clone the repo and follow [README.md](./README.md).
2. For setup issues, see [docs/TROUBLESHOOTING.md](./docs/TROUBLESHOOTING.md).
3. Optional MCP: [docs/MCP-SETUP.md](./docs/MCP-SETUP.md).

## How to help

- **Bugs:** Open a GitHub Issue with steps to reproduce, OS, and Rust/Node versions if relevant.
- **Ideas:** Issues with the `idea` label (or plain description) are welcome. Phase 1 scope is in [ContextLayerPRD.md](./ContextLayerPRD.md).
- **Pull requests:** Small, focused PRs are easiest to review. For larger changes, open an issue first so we do not duplicate work.

## Scope (beta)

In scope: workspaces, blocks (hypothesis / action / evidence / conclusion), workspace health, local SQLite, optional MCP.

Out of scope for now: cloud sync, Obsidian export, pre-built installers for all platforms, auto-ingest from chat.

## Code

- Rust: `crates/`, `apps/mcp-server/`, Tauri backend in `apps/desktop/src-tauri/`
- UI: `apps/desktop/src/`
- Migrations: `migrations/`

Run the desktop app with `npm run dev` from repo root (after `npm run desktop:install` once).

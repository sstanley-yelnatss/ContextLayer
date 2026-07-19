# Contributing to ContextLayer

Thanks for trying ContextLayer. This project is in **friends beta**: APIs and UX may change.

**License:** [AGPL-3.0](./LICENSE). Contributions are accepted under the same license. Design partners: use the **Design Partner Feedback** issue template.

## Branching

| Branch | Purpose |
|--------|---------|
| **`main`** | Stable / demo-ready. Merge via PR only. |
| **`develop`** | Active integration — default target for day-to-day work. |
| **`feature/*`** | Optional short-lived branches off `develop` for larger changes. |

**Flow:** work on `develop` → open PR **`develop` → `main`** for releases/milestones → tag on `main`.

CI runs on pushes and PRs to `main` and `develop`.

## Quick start

**Users:** download the Windows installer from [Releases](https://github.com/sstanley-yelnatss/ContextLayer/releases) — see [README.md](./README.md).

**Contributors:** clone, then:

```powershell
npm run desktop:install
npm run dev
```

Docs: [docs/cheatsheets/TROUBLESHOOTING.md](./docs/cheatsheets/TROUBLESHOOTING.md) · [docs/cheatsheets/MCP-SETUP.md](./docs/cheatsheets/MCP-SETUP.md)

## Pull requests

- Target **`develop`** for features and fixes.
- Paste **PR Reasoning** in the description (see [.github/PULL_REQUEST_TEMPLATE.md](./.github/PULL_REQUEST_TEMPLATE.md)) so Trace CI passes.
- Small, focused PRs are easiest to review.

## Code layout

- Rust: `crates/`, `apps/mcp-server/`, `apps/recorder/`, `apps/trace-cli/`, Tauri backend in `apps/desktop/src-tauri/`
- UI: `apps/desktop/src/`
- Migrations: `migrations/`
- User docs: `docs/cheatsheets/`

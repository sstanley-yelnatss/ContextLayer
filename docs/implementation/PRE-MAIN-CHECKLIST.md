# Pre-main release checklist

**Order:** implement polish below → merge `develop` → `main` → build `.exe` → co-founder demo.

---

## 1. Agent DevOps template + Assumption labeling

- [x] **Rust core** — `agent_devops` on `WorkspaceTemplate`; label **`Agent DevOps`** (capitalized)
- [x] **Desktop types** — `types.ts`: template in union, first in dropdown, `hypothesisFieldLabel()` → Assumption vs Hypothesis
- [x] **UI** — `BlockPanel`, `TimelinePage`: template-aware first-field label (Assumption for Agent DevOps; Hypothesis for Penetration Testing)
- [x] **Export** — PR + workspace + agent context exports: `**Assumption:**` for `agent_devops`, `**Hypothesis:**` for `security_hunt`
- [x] **Tests** — export crate test for Assumption label when template is `agent_devops`

Internal schema stays `hypothesis`; only display/export labels change.

---

## 2. Copy & positioning

- [x] **Pitch** — `docs/PITCH.md`: Agent DevOps / AI change governance primary; assumption language for eng wedge
- [x] **Landing** — `templates/dark-editorial-landing/contextlayer.html`: governance + reasoning receipt hero (not investigator-first)
- [x] **Docs touch** — `docs/MCP-TOOLS.md`, `docs/mcp-cursor-cheatsheet.md` template list includes `agent_devops`

---

## 3. Optional polish (before `.exe`, not blocking merge)

- [ ] README screenshot / demo workspace
- [x] Workspace list header copy toward governance
- [x] Default new workspace template → Agent DevOps (desktop + MCP)

---

## 4. Merge & release

- [ ] Open PR `develop` → `main` — **Trace CI runs automatically** (`.github/workflows/trace-ci.yml` + `.contextlayer/rules.yml`)
- [ ] **Paste ContextLayer PR export** into the PR description (section **PR Reasoning**) so `require_reasoning_export` passes — full export includes `PR Reasoning:`; Agent DevOps block bodies use `Assumption:` (also a valid marker)
- [ ] Confirm Trace CI job green before merge (branch protection may require it)
- [ ] `npm run desktop:build` from **repo root** (builds sidecars + Tauri installer) → ship NSIS setup `.exe`
- [ ] Verify install folder contains `ContextLayer.exe`, `contextlayer-recorder.exe`, `contextlayer-mcp.exe`, `contextlayer-trace.exe`
- [ ] GitHub Release notes (optional) — see [INSTALLER-BUNDLE-PLAN.md](./INSTALLER-BUNDLE-PLAN.md)

---

## 5. After demo (backlog)

- [ ] Branch labels in PR export (optional)
- [ ] Investigator pitch as separate lane (back burner)

---

## Key files

| Area | Paths |
|------|--------|
| Template enum | `crates/core/src/types.rs`, `crates/core/src/admission.rs` |
| UI | `apps/desktop/src/types.ts`, `BlockPanel.tsx`, `TimelinePage.tsx`, `WorkspaceListPage.tsx` |
| Export | `crates/export/src/lib.rs` |
| MCP | `apps/mcp-server/src/main.rs` |
| Copy | `docs/PITCH.md`, `templates/dark-editorial-landing/contextlayer.html` |

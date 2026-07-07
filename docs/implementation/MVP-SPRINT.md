# ContextLayer — True MVP sprint

**Target:** **True merged MVP** by **Friday 2026-06-12** (curated lane tight by Tue 6/9 if possible).  
**Branch:** `develop` → PR → `main` → tags **`v1.3`** (curated) then **`v1.4`** (capture + trace CI)  
**North star:** Full **PR reasoning package** — capture → blocks → export → **trace CI gate** — demoable without hand-waving the governance lane.

---

## What “true MVP” means (merged vision)

**Both layers must work**, not just the reasoning graph:

```
AI session
    ↓  CAPTURE v0 — recorder + checkpoint commits (local trace store)
Raw trace (decision moments, observables, redactable)
    ↓  CURATE — MCP + desktop + import
Blocks + hygiene
    ↓  EXPORT — PR markdown + optional trace attachment path
GitHub PR
    ↓  TRACE CI stub — pass/fail on policy (artifact present, secrets scan, rules.yml)
Reviewer + compliance can trust the package
```

**Curated-only is v1.3.** **Capture + trace CI = required for true MVP (v1.4).**

---

## ~~Fork block UX (T2)~~ — dropped (2026-06)

**Removed from backlog.** Parallel reasoning paths are covered by **new blocks + `link_to_block_ids`** and **capture branch/merge** for conversation tangents. No `forked_from` schema or Fork button planned.

---

## Exit criteria (true MVP — all required)

### Curated lane (v1.3)

- [ ] **E1** Real PR on ContextLayer with reasoning export in PR description
- [ ] **E2** Fresh clone smoke test: workspace → blocks → PR export
- [ ] **E3** Cursor agent logs via MCP with skill (no manual tool names)
- [ ] **E4** Import spike: ≥3 blocks from one past session
- [ ] **E6** Demo workspace: 3–5 block story with rejected path

### Capture + governance lane (v1.4 — true MVP)

- [ ] **E7** **Capture v0:** session events append to local trace store under `~/.contextlayer/traces/` (or DB table); **checkpoint commit** API (intent + note + rejected paths + optional `git_sha`)
- [ ] **E8** Checkpoints **link** to `workspace_id` (and optionally block IDs) — no duplicate SQLite checkpoint index table
- [ ] **E9** **Trace CI stub:** GitHub Action (or `cargo run -p contextlayer-cli trace check`) fails PR if: (a) no reasoning artifact per rules, (b) secrets pattern in trace/export, (c) optional “checkpoint before merge” rule
- [ ] **E10** **`.contextlayer/rules.yml`** stub checked into repo — document 2–3 rules (e.g. `require_reasoning_export: true`, `require_checkpoint: false` for v0)
- [ ] **E11** Dogfood PR passes trace CI (or documents waivable failure with reason)

### Release

- [ ] **E5** `v1.3` / `v1.4` tagged when exit criteria met (code + user-facing docs only in release PR)

---

## P0 — Curated lane (Sun–Tue)

### Release hygiene (~30 min)

- [ ] `cargo test` + desktop smoke in Tauri window
- [ ] Fix anything broken before new features

**Internal docs** (`MVP-SPRINT.md`, `EVAL-PAPER.md`, competitive/planning notes): **keep local** — not committed to the public repo. Update **user-facing** docs only inside feature PRs (`MCP-TOOLS.md`, README capability bullets when something ships).

### T3 — Agent context compile (~3–4 hr)

- [ ] `compile_agent_context_markdown` in `crates/export`
- [ ] MCP `compile_agent_context` + desktop “Copy for agent”
- [ ] Update `docs/MCP-TOOLS.md`

### G1 — Tiered MCP index (~2–3 hr)

- [ ] MCP `get_workspace_index` (no body text)
- [ ] Tier docs in MCP-TOOLS + cheatsheet

### G5 — Cursor skill (~1–2 hr)

- [ ] `.cursor/skills/contextlayer/SKILL.md` + README link

### O1 — Session import spike (~4–6 hr)

- [ ] `import_session` (paste or JSON) → workspace + draft blocks
- [ ] Tag imported blocks `imported` / needs_review

### P1 polish (Tue)

- [ ] Demo workspace fixture
- [ ] G3 lite toast after export
- [ ] B2 lite branch/PR# in export header
- [ ] Dogfood PR + example `docs/reasoning/` artifact
- [ ] Tag **v1.3** if curated exit criteria met

---

## P0 — Capture v0 (Wed–Thu) — true MVP core

*Single trace store; checkpoints live here (G2).*

### Design (Wed AM, ~2 hr)

- [ ] `docs/CAPTURE-V0.md` — trace file format, checkpoint commit schema, link to workspace/blocks, redaction hooks

### Implementation (~1–1.5 days)

- [ ] New crate or module: `crates/trace` (or `crates/capture`)
- [ ] Trace store: append-only JSONL per workspace **or** `traces` SQLite table (events + checkpoint rows)
- [ ] **Recorder v0** (pick one for MVP):
  - **A)** MCP `append_trace_event` + `commit_checkpoint` (agent-driven capture — ships fastest)
  - **B)** File watcher on Cursor transcript export folder (if reliable path exists)
  - **C)** Manual “Import session” writes to trace **and** blocks (O1 extended)
- [ ] Checkpoint commit: `{ intent, note, rejected_paths[], git_sha?, workspace_id, block_ids?[] }`
- [ ] MCP: `list_checkpoints`, `get_trace_summary` (tiered — index only by default)
- [ ] Desktop: “Checkpoint this decision” button on workspace (writes trace + optional link to selected blocks)
- [ ] Redaction stub: strip API key patterns on write (regex list in rules.yml)

### Import + capture overlap

- [ ] O1 import also appends raw transcript slice to trace store (one pipeline, two views)

---

## P0 — Trace CI stub (Thu–Fri) — true MVP core

*No GitHub App required — Action + local CLI is enough for v0.*

### Rules file

- [ ] `.contextlayer/rules.yml` in repo root (example + schema doc)
- [ ] Parser in Rust (`crates/trace` or `crates/ci`)

### Checks (v0 minimum)

| Rule | Check |
|------|--------|
| `require_reasoning_export` | PR body or `docs/reasoning/*.md` contains ContextLayer export marker OR min section headers |
| `secrets_scan` | Fail if trace/export matches key patterns |
| `require_checkpoint` | Optional off by default; if on, trace has ≥1 checkpoint linked to workspace |

### CI wiring

- [ ] `.github/workflows/trace-ci.yml` — runs on PR; `cargo run -p contextlayer-cli -- trace check` (or dedicated binary)
- [ ] Local: same command for pre-push dogfood
- [ ] PR template: section for reasoning export + link to trace path

### Dogfood

- [ ] ContextLayer repo’s own PRs run trace CI
- [ ] Tag **v1.4** when E7–E11 pass

---

## P2 — Stretch / polish

- [ ] EVAL-PAPER first dogfood row
- [ ] Windows `tauri build` / unsigned installer
- [ ] `list_workspaces` MCP tier-0
- [ ] GitHub App / bot comment (nice, not required for trace CI)

---

## P3 — After true MVP

| Item | Notes |
|------|--------|
| ~~Fork block UX (T2)~~ | **Dropped** — use block links + capture branch |
| Dual-panel UI (T5) | |
| Full multi-format import | |
| Hosted read-only share (O2) | |
| Eval paper publish | |
| Enterprise self-host | |

---

## Revised day plan

| Day | Focus |
|-----|--------|
| **Sun 6/7** | T3 + G1 start (skip doc commits) |
| **Mon 6/8** | T3, G1, G5, O1 |
| **Tue 6/9** | Curated polish, dogfood PR, **v1.3** |
| **Wed 6/10** | CAPTURE-V0 design + trace store + MCP checkpoint tools |
| **Thu 6/11** | Recorder path (MCP-first); desktop checkpoint; import→trace |
| **Fri 6/12** | Trace CI Action + rules.yml; dogfood PR passes CI; **v1.4 true MVP** |

---

## Build order (full thread)

1. Green CI smoke test  
2. T3 → G1 → G5 → O1 (curated)  
3. Demo + dogfood PR → **v1.3**  
4. CAPTURE-V0 design  
5. Trace store + checkpoint commits + MCP  
6. Desktop checkpoint + import→trace  
7. rules.yml + trace check CLI  
8. GitHub Action → dogfood → **v1.4**  

---

## Risk flags

| Risk | Mitigation |
|------|------------|
| Tue deadline slips | **v1.3** Tue, **v1.4** Fri — two tags, one sprint |
| Cursor auto-capture hard | **MCP-first recorder** for v0; file watcher later |
| Trace CI too strict | Start with `require_reasoning_export` only; secrets scan warn-only first |

---

## Changelog

| Date | Change |
|------|--------|
| 2026-06-07 | Initial sprint — curated-only Tue target |
| 2026-06-07 | **Revised:** true MVP = capture v0 + trace CI; v1.3/v1.4 split; fork UX glossary |
| 2026-06-07 | Internal planning docs stay local; no standalone docs commit |
| 2026-06-07 | **T2 fork block UX dropped** — block links + capture branch cover parallel paths |

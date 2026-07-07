# ContextLayer PRD Addendum — Merged product vision (ContextLayer × GitLLM)

**Version:** 1.0  
**Status:** Active — extends locked parent; co-founder review pending on naming & GitHub App ownership  
**Parent:** [ContextLayerPRD.md](../ContextLayerPRD.md) v1.0  
**Related:** [PRD-addendum-blocks.md](./PRD-addendum-blocks.md) · [POSITIONING.md](./POSITIONING.md) · [FUTURE-IMPLEMENTATION.md](./FUTURE-IMPLEMENTATION.md) §8  
**Last updated:** 2026-06-07

---

## 0. Why this addendum exists

[ContextLayerPRD.md](../ContextLayerPRD.md) v1.0 is **locked** for Phase 1 reasoning-graph invariants. Since lock:

- **Blocks** became the primary UX unit ([PRD-addendum-blocks.md](./PRD-addendum-blocks.md))
- **MCP** shipped (dogfood logging lane — parent PRD §2 incorrectly listed MCP as out of scope)
- **B1 PR export** shipped (multi-select blocks → PR markdown)
- Co-founder **GitLLM** thesis was merged into one platform direction (capture + governance + structured reasoning)

This addendum is the **current product scope** for investors, co-founder alignment, and implementation planning. It does **not** replace core schema invariants in the parent PRD.

---

## 1. Problem (merged)

| Pain | Detail |
|------|--------|
| **PR volume** | AI multiplies PRs; reviewers see diffs, not the path that produced them |
| **Context trap** | Reasoning stuck in 1:1 dev↔agent chats; team can't audit or reuse |
| **Unstructured memory** | Notes and chat logs don't expose open loops, contradictions, or dead paths |
| **Compliance gap** | Enterprises need observable engineering evidence — not hidden chain-of-thought, not raw prompt dumps |

---

## 2. Vision (merged one-liner)

> **Git tracks what changed. We track the auditable, reviewable reasoning path that produced the change — raw trace for compliance, structured graph for humans.**

**Dual pitch (both active):**

| Lane | Audience | Wedge |
|------|----------|-------|
| **Investigator** | Security research, debugging, strategic decisions | Structured local investigations + hygiene |
| **Agent DevOps** | Eng teams with AI-assisted PR volume | Reasoning receipt on every PR |

---

## 3. Architecture — two layers

```
┌─────────────────────────────────────────────────────────────┐
│  AI session (Cursor, Claude Code, Codex, …)                 │
│       ↓ capture (roadmap: Agent Trace / checkpoint recorder)  │
│  Raw trace — audit, local, redacted                         │
│       ↓ curate (MCP + desktop today)                        │
│  ContextLayer workspace — blocks, H/A/E/C, hygiene          │
│       ↓ export (B1 — shipped)                               │
│  PR markdown artifact → GitHub PR description / comment     │
│       ↓ governance (roadmap: trace CI, rules.yml)           │
│  Policy checks — secrets redacted, trace present, goal stated │
└─────────────────────────────────────────────────────────────┘
```

| Layer | Owner lane | Storage | Phase |
|-------|------------|---------|-------|
| **Curated reasoning** | ContextLayer (Miles) | SQLite `~/.contextlayer/graph.db` | **Now** |
| **PR export compile** | ContextLayer | Markdown (derived view) | **Now** (v1.2) |
| **MCP read/write** | ContextLayer | Same SQLite | **Now** |
| **Raw session capture** | GitLLM / capture module | Agent Trace-compatible blobs, git-adjacent | Phase 2–3 |
| **GitHub App + PR bot** | Co-founder lane | Hosted integration | Phase 2–3 |
| **Trace CI / policy** | Shared | `.gitllm/rules.yml` style | Phase 3 |

**Agent Trace:** Optional interchange format for line-level attribution. **Not** the product. See [POSITIONING.md](./POSITIONING.md).

---

## 4. Core product invariants (unchanged from parent)

- Four typed primitives: hypothesis, action, evidence, conclusion
- Constraint is **not** first-class
- Conclusion admission: ≥1 hypothesis + ≥1 evidence when conclusion is saved
- AI is **not** system of record; user-owned deterministic graph
- Compile outputs (markdown) are **views** — never shadow schema

---

## 5. Shipped scope (through v1.2)

| Capability | Status |
|------------|--------|
| Workspaces + templates | ✅ |
| Blocks (primary UX) + belief states + system tags | ✅ |
| Workspace hygiene panel | ✅ |
| Desktop app (Tauri + React) | ✅ |
| MCP server (20 tools — see [MCP-TOOLS.md](./MCP-TOOLS.md)) | ✅ |
| Multi-block PR export (desktop + MCP `export_blocks`) | ✅ |
| Copy full workspace summary | ✅ |
| CI (`cargo test` core + MCP) | ✅ |
| `develop` / `main` branch workflow | ✅ |

---

## 6. Phase roadmap (updated)

| Phase | Focus | Includes |
|-------|--------|----------|
| **1.x (NOW)** | Reasoning graph + PR wedge | Blocks, hygiene, MCP, PR export, dogfood |
| **1.5** | LLM workspace summary | Suggest/classify only; never auto-commit |
| **2** | Git-native + PR metadata | Committed reasoning artifact; PR ↔ workspace link; optional hosted read-only viewer |
| **2.5** | Capture layer v1 | Checkpointed raw trace (not every prompt); Cursor-first recorder |
| **3** | GitHub integration | App/bot: PR comment + link; trace CI stub; redaction |
| **3.5** | Governance | Policy-as-code, reasoning CI, enterprise self-host path |
| **4** | Cross-AI continuity | Same workspace across tools; compile injection packets |
| **5** | Broad platform | Team fork/join conversations, public repos — **explicitly not v1** |

---

## 7. Merged MVP wedge (single ship target)

> **PR reasoning package:** GitHub PR + ContextLayer export (selected blocks) + optional raw trace attachment + pass/fail trace CI stub.

**Workflow (target end state):**

1. Dev works in Cursor; MCP logs blocks to PR-anchored workspace
2. Dev selects blocks → export markdown (✅ today)
3. Recorder captures observable trace; dev squashes to checkpoints (roadmap)
4. Open PR → summary + links + CI (roadmap)
5. Reviewer reads structured export; optionally imports context into own agent (roadmap)

---

## 8. Non-goals (merged — do not drift)

- Every prompt = commit (use **checkpoints** in raw layer, **blocks** in curated layer)
- "Git for chats" consumer pitch (enterprise governance framing instead)
- Hidden chain-of-thought storage
- Public "open source conversations" social network (stage 5 at earliest)
- Replacing Linear, Jira, or Copilot Code Review
- Replacing Langfuse/LangSmith for LLM app observability
- Graph visualization as v1 requirement
- Cloud sync / multi-user auth in v1

---

## 9. Storage architecture (decided)

| Layer | Role |
|-------|------|
| **SQLite** | Canonical structured graph; hygiene; belief history; MCP R/W |
| **Markdown** | Compiled export only — PR artifacts, human review |
| **Raw trace (future)** | Audit blobs; Agent Trace-compatible; linked by PR ID |

Do **not** replace SQLite with markdown-primary storage.

---

## 10. Open decisions (co-founder sign-off)

1. Final product name (ContextLayer vs TraceForge vs GitLLM umbrella)
2. GitHub App + self-host ownership split long-term
3. Emit Agent Trace records in v1 vs v2
4. When to promote this addendum sections into parent PRD v2.0

---

## 11. Success metrics (directional)

| Metric | Phase 1.x |
|--------|-----------|
| Dogfood | 2+ workspaces actively used weekly |
| PR wedge | ≥1 real PR shipped with ContextLayer export attached |
| Reviewer | Reviewer can explain *why* without opening AI chat |
| MCP | Agent can log full block without manual copy-paste |
| Hygiene | User acts on ≥1 hygiene signal per session |

---

## Changelog

| Date | Change |
|------|--------|
| 2026-06-07 | Initial merged-vision addendum; v1.2 shipped scope; co-founder GitLLM integration |

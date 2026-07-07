# ContextLayer — Future implementation backlog

**Status:** Living doc — ideas captured after public beta + startup demo (Jun 2026).  
**Not committed to timeline.** Implementation order TBD after co-founder review and design partners.

**Related (locked product spec):**

- [ContextLayerPRD.md](../ContextLayerPRD.md) — v1.0 invariants
- [PRD-addendum-blocks.md](./PRD-addendum-blocks.md) — Phase 1.1–1.2 done; Phase 1.3+ schema/features
- **[MVP-SPRINT.md](./MVP-SPRINT.md)** — **active build list** (true MVP target ~2026-06-09)

This file tracks **new wedges and GTM-driven work** that extend beyond the original personal-tool / investigator scope.

---

## 0. Positioning (dual pitch)

Keep both angles active; do not drop either.

| Pitch | Audience | One line |
|-------|----------|----------|
| **Investigator** | Bug bounty, security research, debugging SWEs | Structured local investigations with hygiene for open loops |
| **Agent DevOps** | Eng teams drowning in AI-assisted PR volume | **Reasoning receipt** attached to a PR so reviewers see *why*, not just *what* changed |

**Category (emerging):** AI change governance / PR reasoning / human-in-the-loop review for agent-assisted development.

**Complement, not compete:**

- **Graphify** → what the *code* is and what a PR *touches* (structure)
- **Linear** → what *work* exists and what *shipped* (issues, releases)
- **ContextLayer** → why the author is *confident* in the change (epistemic timeline)

---

## 1. PR reasoning export (B1) — priority candidate

### Problem

Reviewers see a diff without the **reasoning timeline** behind it. AI-heavy teams produce more PRs; approval bottleneck is trust and context, not just code review capacity.

### MVP flow (build toward)

1. In workspace timeline, **multi-select blocks** (one or many).
2. **Export for PR** → compiled **markdown artifact** (SQLite remains source of truth).
3. Author attaches artifact to PR (description, comment, or committed file — see §2).

One workspace may contain **multiple fixes/issues** across separate blocks; one PR may need context from **several selected blocks**.

### Artifact contents (MVP)

- Workspace name + **goal**
- Selected blocks in **timeline order**, each with:
  - Title
  - Hypothesis / action / evidence / conclusion (verbatim)
  - Belief state, system tag, user tag
  - Conclusion outcome, decision tag, confidence (if present)
- Optional footer: hygiene snapshot for selection (“N blocks still open in this export”)

### Format

- **Primary:** Markdown (GitHub-native, human-readable, LLM-friendly as a *slice*)
- **Optional later:** JSON for bots / CI

### Existing code to extend

- `crates/export` — workspace summary compile today; extend to **selected block IDs** (see **§1.1**)
- Desktop: multi-select UI on timeline
- MCP: optional `export_blocks` tool for agent-driven PR prep

### §1.1 Plain English — what “extend export for selected block IDs” means

**Today:** “Copy summary” exports **every block** in the workspace. One workspace might hold a whole investigation (10+ blocks across multiple bugs or ideas). That’s fine for a full report; it’s wrong for a PR, which usually only needs **the blocks that explain this change**.

**Block ID:** Each block has a stable UUID in SQLite (shown in export as `` `abc-123...` ``). The app already uses these for linking blocks.

**What we add in Rust (`crates/export`):**

```text
compile_workspace_summary_markdown(store, workspace_id)
  → all blocks (unchanged; keep for “Copy full summary”)

compile_pr_export_markdown(store, workspace_id, block_ids: &[String])
  → workspace header (name, goal)
  → ONLY blocks whose id is in block_ids
  → PR-oriented footer (optional: “3 blocks selected, 2 open loops not included”)
```

No schema change — read-only filter on blocks already fetched from the DB.

**What we add in the desktop app:** Checkboxes on the timeline (or “Export for PR” mode) so you pick blocks → button calls the new function with those IDs → copy to clipboard.

**Why it matters for demo:** You can show a messy workspace with 8 blocks, select 3 that map to “the PR,” export a tight reasoning receipt — that’s the Agent DevOps vision in one click.

### §1.2 Phase 1 sprint — demo tomorrow (YC friend)

**Minimum to show vision (prioritize in this order):**

| Priority | Ship | Demo moment |
|----------|------|-------------|
| P0 | Pre-loaded or live workspace with **realistic PR story** (3–5 blocks: hypothesis → rejected path → chosen fix → conclusion) | “This is the investigation behind the change” |
| P0 | **Export for PR** with multi-select OR “export all” with a clean workspace dedicated to one PR | Paste markdown in browser/GitHub — reviewer reads *why* |
| P1 | Hygiene panel + one orphan/stale example | “Open loops don’t get lost” |
| P1 | MCP live: agent adds evidence to a block by title | “Works inside Cursor while you code” |
| P2 | B2 lite metadata in export header (branch, PR #) — nice, not required tonight | |

**Fallback if time runs out:** Full-workspace export already works (`Copy summary`). Use a **single-purpose demo workspace** with only PR-relevant blocks so export is already a “reasoning receipt.” Multi-select is still the right B1 build, but don’t block the demo on it if one clean workspace tells the story.

**Exit criteria (unchanged):** Someone who wasn’t in your chat can read the export and understand why the change was made.

### §1.3 Phases 2 & 3 — queued immediately after Phase 1

Do not start until Phase 1 exit criteria met (export + one dogfood or demo-quality artifact).

**Phase 2 — Git-native handoff**

- Document commit path: `.contextlayer/pr-N-reasoning.md` or `docs/reasoning/`
- Sample committed artifact in repo for demos
- 1–2 friends try attach-on-real-PR workflow

**Phase 3 — Capture layer (co-founder lane; Miles executes)**

- Checkpoint model design doc
- Local session recorder → checkpoints (not every prompt)
- Redaction / `.contextlayerignore`
- Optional Agent Trace JSON linking to reasoning export URL
- Cursor-first auto-capture where hooks exist

See also §8 solo execution roadmap in merge section.

### §1.4 PR handoff — delivery options & reviewer discoverability

**Concern:** If the artifact is only a file buried in the PR **Files changed** tab, reviewers may miss it. Valid — discoverability is a product decision, not just storage.

| Phase | Method | Reviewer sees it | Notes |
|-------|--------|------------------|-------|
| **1 (demo)** | Paste into **PR description** | First screen when opening PR | Zero infra; stands out |
| **2** | Commit `docs/reasoning/pr-N.md` on branch | Files tab + optional link in description | Git-native; add **PR template** line: “Link or paste reasoning export” |
| **2** | `gh pr edit --body-file` / script | Description | CLI-friendly without App |
| **3** | GitHub Action / bot **comment** on PR open | Conversation tab; can pin | Artifact visible without digging diffs |
| **4** | Required status check + PR template | Block merge if missing | Enterprise |

**Default recommendation for teams:** Description **or** pinned bot comment linking to committed file — not file alone without a pointer in the description.

**Tonight:** paste in description only.

### Later (not MVP)

- GitHub Action / bot: post artifact as PR comment on open/update
- Metadata: link workspace ↔ PR number ↔ branch name
- “Mark blocks for this PR” tag or filter
- Sample artifact from dogfood workspace for sales demos

### Acceptance criteria (draft)

1. User can select 1+ blocks and copy/save markdown export.
2. Export includes only selected blocks, ordered by `updated_at` or user-defined order.
3. No SQLite schema change required for v1 export (read-only compile).
4. Export path documented for “paste into PR description” workflow.

---

## 2. Collaboration & PR anchoring (B2)

### Problem

Reviewer may need to **challenge reasoning**, add evidence, or mark needs_review in the **same** graph — not only read a static export.

### Agreed direction (sequencing)

| Phase | Approach | Notes |
|-------|----------|--------|
| **Now / beta** | **(4) Git-native artifact** | Export markdown → PR description or commit under e.g. `docs/reasoning/pr-42.md` or `.contextlayer/pr-42-reasoning.md`. Versioned with code; no sync backend. |
| **Next** | **(3) PR-anchored workspace** | Workspace object tied to PR `#42` / branch; author works in app; export stays the handoff surface. |
| **Later** | **Hosted / shared workspace** | Multi-user access; reviewer can edit/add blocks; requires auth, sync, conflict model — **explicitly deferred**. |

### Open design (when ready)

- Real-time co-editing vs async comments vs fork-and-merge workspace
- Read-only share link vs full collaborator
- Enterprise: SSO, audit log, repo/org scoping

### Do not implement until

- B1 export dogfooded on real PRs
- At least one design partner confirms workflow

---

## 3. Graphify-inspired ideas (B3)

**Reference:** [graphify](https://github.com/safishamsi/graphify) — codebase knowledge graph + `graphify prs` (blast radius, graph communities, merge-order conflicts, AI triage).

**Steal (adapt to reasoning graph, not codebase graph):**

| Idea | ContextLayer adaptation |
|------|-------------------------|
| **PR as first-class object** | Deep link / view: “PR #42 + linked reasoning blocks + hygiene flags” |
| **Blast radius metaphor** | “Reasoning surface area”: how many open hypotheses, thin evidence, or unsettled conclusions touch this PR |
| **Conflict / overlap detection** | Two open PRs share unresolved hypotheses or contradicting conclusions in same workspace |
| **Review queue triage** | Rank PRs by reasoning debt (orphans, stale, needs_review in linked blocks) — hygiene → triage |
| **MCP PR tools** | e.g. `export_blocks_for_pr`, `get_pr_reasoning_summary` (after GitHub integration exists) |
| **Dashboard mental model** | Single screen: PR metadata + selected blocks + health counts |

**Do not become:**

- AST / codebase extraction graph
- Replacement for Cursor/Graphify-style repo memory
- “grep killer” for source files

**Positioning reminder:** Graphify = agent understands the **repo**. ContextLayer = team understands **why the change was made**.

---

## 4. Linear-inspired ideas (B3)

**Reference:** [Linear](https://linear.app) — issues, agent delegation, Code Intelligence, MCP, Releases, review sync to GitHub.

**Steal (UX and workflow patterns):**

| Idea | ContextLayer adaptation |
|------|-------------------------|
| **Clean, fast timeline UI** | Density, keyboard nav, minimal chrome — keep improving desktop timeline |
| **Delegate vs assign** | Human owns workspace/PR; agent (MCP) assists logging — mirror “human responsible, agent acts” |
| **Agent / team guidance** | Workspace templates + checklist: “hypothesis logged before PR export” |
| **Release notes from structured objects** | Block export ≈ “generate reasoning summary from selection” (parallel to Linear release notes from issues) |
| **Official MCP as integration surface** | Enterprise expects MCP; expand tools deliberately |
| **Status as visual language** | Belief states + system tags stay **epistemic**, not generic “In Progress” |
| **Code / issue context (optional future)** | Link block to Linear issue ID or GitHub issue — **integration, not replacement** |

**Do not become:**

- Full issue tracker or sprint tool
- Jira/Linear competitor
- Primary home for “what work exists”

**Positioning reminder:** Linear tracks **what** shipped. ContextLayer tracks **why we’re confident** it should ship.

---

## 5. Storage architecture (B4) — decided

| Layer | Role |
|-------|------|
| **SQLite** | Canonical source of truth; hygiene queries; belief history; MCP read/write |
| **Markdown** | **Compiled export only** — PR artifacts, human review, optional agent read of one slice |

Do **not** replace SQLite with markdown-primary storage. Benchmark token use via MCP tools vs full-file read when skeptics ask; expect selective tools to win for large workspaces.

---

## 6. Parking lot (from demo — not scoped)

- One-pager: dual pitch + complement Graphify/Linear
- Token benchmark write-up for investors/reviewers
- Screenshot / demo script for Agent DevOps angle
- Issue labels on GitHub: `bug`, `question`, `idea`, `agent-devops`

---

## 9. Competitive steals — GCC, Twigg, OneContext (Jun 2026)

**Status:** Captured from co-founder competitive review. **Agreed items marked ✅.** Prioritize after reviewing this table together — not auto-scheduled.

**Pitch adjustment (✅ note):** Add competitive one-liner vs chat-log tools:

> **Others version chat or agent logs. We version reasoning — and ship it to the PR.**

See [POSITIONING.md](./POSITIONING.md) § Competitive differentiation.

### 9.1 GCC (Graph Context Compiler) — agreed steals

| ID | Idea | Status | Plain English | Implementation sketch |
|----|------|--------|---------------|------------------------|
| **G1** | **Tiered context retrieval** | ✅ **Scheduled — future implementation** | Agent asks for context in **layers**, not one giant dump. **Layer 0:** workspace list + goals (50 tokens). **Layer 1:** block titles + belief/hygiene flags only (cheap scan). **Layer 2:** full block body for 1–3 IDs you actually need. **Layer 3:** compiled PR export slice. Today MCP has `list_blocks` / `get_block` / `export_blocks` — formalize as **documented tiers** + new `get_workspace_index` tool that never returns full evidence text. | **Build:** MCP `get_workspace_index(workspace_id)` → `{ goal, blocks: [{ id, title, type, belief, hygiene_flags }] }`. Document tier protocol in MCP-TOOLS + agent skill. Token benchmark in [EVAL-PAPER.md](./EVAL-PAPER.md) Arm B / H4. |
| **G2** | **Decision-only checkpoints** | ✅ **Defer to capture lane — no separate SQLite table** | Checkpoints = **decision moments in the raw trace**, not every prompt. **Do not** build a parallel `checkpoints` table in SQLite first — it duplicates blocks today and becomes throwaway when capture ships. **Blocks = curated checkpoint layer now.** When capture lands, checkpoint commits live in the trace store and **link** to `workspace_id` / block IDs. | Capture recorder writes checkpoint commit (intent, rejected paths, git_sha, observables). MCP/app: `list_checkpoints` reads from trace store, not a second schema. |
| **G3** | **Proactive checkpoint prompts** | ✅ Do it — **blocks/export only until capture** | After milestone (tests pass, PR opened, conclusion block), nudge: “Add a conclusion block?” / “Export for PR?” — **not** a separate checkpoint entity. When capture exists, same nudge can also write a trace checkpoint. | Desktop toast after export; optional skill hook. No new tables for v1 prompts. |
| **G4** | **Worktree TTL / stale branch cleanup** | ✅ Do it | Hygiene for **git** tangents: flag worktrees/branches with no activity N days; offer cleanup. Maps to “clean work tree before PR.” | Extend hygiene panel: link workspace → branch name metadata (B2); cron/local job lists stale branches; not AST — **reasoning debt + branch debt** together. |
| **G5** | **Cursor skill alongside MCP** | ✅ Open — design needed | Skill = **when/how** doc for agents; MCP = **API**; desktop = **human UI**. All share `~/.contextlayer/graph.db`. See §9.5. | Ship `.cursor/skills/contextlayer/SKILL.md` in repo + docs; skill references MCP tool names; no duplicate storage. |
| **G6** | **Study OSS repo** | ✅ Agent + Miles (not co-founder) | Read GCC implementation for tiered retrieval + benchmark methodology before building G1/G10. | Time-boxed skim; agent summarizes patterns into this doc changelog. |

### 9.2 Twigg — agreed steals

| ID | Idea | Status | Notes |
|----|------|--------|-------|
| **T1** | **“Context rot”** marketing term | ✅ | Use in site/README: long sessions lose thread; structured blocks + hygiene fight rot. |
| **T2** | ~~**Fork block UX**~~ | **Dropped** | Redundant with `link_to_block_ids` + capture branch/merge. Not building `forked_from` / Fork button. |
| **T3** | **Compile context packet for agent** | ✅ — **may move before Phase 4** | One-click “send this workspace slice to agent” = structured markdown/json packet (selected blocks + goal + hygiene). Phase 4 in old plan = capture; **this is curated-layer export for agent consumption** — can ship right after B1 as `compile_agent_context`. |
| **T5** | **Dual-panel exploration UI** | ✅ if done right | Timeline + block detail side-by-side. Desktop UX pass — don’t block MCP wedge. |

### 9.3 OneContext — agreed steals

| ID | Idea | Status | Notes |
|----|------|--------|-------|
| **O1** | **Import Cursor/Codex/Claude sessions → blocks** | ✅ **Scheduled — import spike** | **Steal the *feature idea* from OneContext — build it into ContextLayer.** See **§9.4**. |
| **O2** | **Share read-only workspace link** | ✅ | Deferred until hosted sync; v0 = export markdown + gist/file; v1 = signed read-only URL. |
| **O3** | **Resume session messaging** | ✅ | Copy: “Pick up where you left off” — workspace list shows last activity + open hygiene count. |

### 9.4 Session import spike (OneContext-style — built into ContextLayer)

**Not OneContext the product.** OneContext is a competitor that imports AI chat sessions into their app. **O1 = we build the same capability inside ContextLayer** so past Cursor/Claude sessions become blocks in *your* graph — no dependency on their service.

**What “spike” means:** Short, throwaway **prototype** (2–4 days) to learn if import is worth full product work. In startup jargon, a spike = time-boxed experiment, not a feature launch.

**Goal:** Prove one real Cursor (or Claude Code) session can become a ContextLayer workspace with **≥3 sensible blocks** without hand-writing every field.

**Steps:**

1. **Pick one format** — Cursor composer export, local chat JSON, or pasted transcript (simplest first).
2. **Heuristic mapper** — user question → hypothesis; tool run / file edit → action; command output → evidence; assistant wrap-up → conclusion (draft belief = `unsettled`).
3. **Import command** — CLI or MCP `import_session(path, workspace_name)` → creates workspace + blocks with metadata `source: import_v0`.
4. **Human review gate** — imported blocks land as `needs_review`; hygiene flags “imported — verify belief.”
5. **Exit criteria:** You run spike on your own last session; co-founder can skim imported timeline and say “I’d use this as starting point.”

**If spike fails:** Still win with MCP live logging (current path) — import is accelerator, not dependency.

### 9.5 Cursor skill + desktop app + SQLite — how they connect

**Does the skill mean less “hey log that as hypothesis”?**  
**Mostly yes for the agent path — not magic, not zero human input.**

| Who | Today | With skill + MCP |
|-----|--------|------------------|
| **You → Cursor agent** | You say “log that as hypothesis” or agent forgets | Skill tells agent: after investigating, call `add_block` with type hypothesis; after test output, log evidence — **without you naming every MCP tool** |
| **You → desktop app** | Manual block creation | Unchanged — skill doesn’t replace desktop |
| **Full automation** | — | **Not v1.** Agent still misses things; you curate in desktop, hygiene catches gaps. Import spike (§9.4) backfills *past* chat; skill improves *live* logging |

So: skill **reduces repetitive prompting** and standardizes agent behavior; it does **not** remove the curated layer or PR export selection.

```
┌──────────────────┐     ┌──────────────────┐     ┌─────────────────────────┐
│  Desktop (Tauri) │     │  MCP server      │     │  Cursor Skill (markdown) │
│  Human curation  │     │  Agent API       │     │  When/how conventions    │
│  Timeline/hygiene│     │  20 tools today  │     │  “Log H before fix” etc. │
└────────┬─────────┘     └────────┬─────────┘     └────────────┬────────────┘
         │                        │                            │
         └────────────────────────┼────────────────────────────┘
                                  ▼
                    ~/.contextlayer/graph.db  (SQLite, single source of truth)
```

- **Skill does not embed SQLite** — it teaches the agent to call **existing MCP tools**.
- **Desktop remains the reviewer/author UI** for hygiene, multi-select PR export, capture controls.
- **Distribution:** Skill ships in repo (`.cursor/skills/…`); MCP config in README; desktop installer optional for non-Cursor users.

**Checkpoints — one system, not two (revised Jun 2026):**

You were right to push back. Building a SQLite checkpoint **index** before capture would mostly duplicate what **blocks** already are, then get superseded when the raw trace layer ships.

| Layer | What it is | When |
|-------|------------|------|
| **Today** | **Blocks** = curated decision timeline (hypothesis → evidence → conclusion) | Shipping |
| **Today** | **G3 prompts** → nudge new blocks or PR export | Can ship without capture |
| **Next** | **Capture** = raw session + checkpoint **commits** at decision moments (co-founder vision) | Phase 3 — single source for audit |
| **Not building** | Separate `checkpoints` table as interim index | Avoid redundant schema |

```
AI session (Cursor, …)
    ↓  capture (build this — checkpoint commits live HERE)
Raw trace — decision-only checkpoints, observables, redaction
    ↓  curate (blocks + MCP — already shipping)
ContextLayer blocks in SQLite
    ↓  export
PR markdown artifact
```

Trace checkpoints **link to** workspace/blocks; they don’t replace blocks. Reviewers read blocks export; compliance reads trace when needed.

### 9.6 Priority stack (suggested — review together)

| Order | Item | Rationale |
|-------|------|-----------|
| 1 | **T3** agent context compile | Extends shipped B1; immediate dogfood value |
| 2 | **G1** tiered MCP index | Cheap win; supports token story + eval paper |
| 3 | **O1** session import spike | ✅ Agreed — onboarding; backfill past chat |
| 4 | **Capture v0 design + minimal recorder** | Checkpoints belong here (G2), not a duplicate SQLite table |
| 5 | **G3** milestone prompts | Block/export nudges; extend to trace checkpoint when capture exists |
| 6 | **G5** Cursor skill | After G1 + T3 stable |
| 7 | **G4** branch TTL | After B2 PR metadata |
| 8 | **O2/O3** share + resume copy | GTM polish |

~~**T2** fork block UX~~ — dropped (Jun 2026); use block links + capture branch instead.

---

## 10. Evaluation paper — GCC-style “with vs without” (✅ wanted)

**Living writeup:** [EVAL-PAPER.md](./EVAL-PAPER.md) — add data, notes, and draft sections there.

**Goal:** Credible artifact like GCC’s SWE-bench analysis — show ContextLayer improves **review comprehension** and **reasoning completeness**, not raw code pass rate alone.

**Arms:** Reviewer wedge (primary) + agent/SWE-adjacent (secondary) — **both**, agreed.

Details, metrics, run logs, and draft outline → **[EVAL-PAPER.md](./EVAL-PAPER.md)**.

**Owner:** Miles + agent co-author; co-founder reviews async.

---

## 11. ICP & validation — mentor feedback (Jun 2026)

**Mentor view (summary):** Proficient SWEs don’t need a reasoning graph to review a PR; many build personal “publisher” agents that gather context and log hypothesis/action/evidence themselves.

**Our read — he’s partially right:**

| Segment | Fit | Why |
|---------|-----|-----|
| **Senior SWE, solo, high agency** | **Weak** | DIY agents + git blame + PR description enough; hardest to sell |
| **Semi-technical builders** (Harvard friend archetype) | **Strong** | Need structure without building infra; hygiene prevents lost threads |
| **AI-heavy teams, compliance** | **Strong** | Audit trail + PR artifact beats private chat |
| **Security / investigator pitch** | **Medium** — backseat for now | Same graph works; GTM focus on Agent DevOps first |
| **Junior–mid devs on agent-heavy teams** | **Medium** | Institutional pattern beats ad-hoc chat logging |

**Publisher agents as competition:** Real for **power users**. Our bet: **structured graph + hygiene + PR export + multi-tool MCP** beats fragile personal scripts that break when they switch models or teammates join. Enterprise won’t adopt “Bob’s publisher script.”

**Validation before going deep:**

1. **5 design partners** who are *not* senior SWEs — semi-technical founders, security students, AI-heavy startup eng.
2. **Success signal:** They export to a real PR twice without you nagging; a reviewer says export helped.
3. **Kill signal:** Partners revert to paste-in-chat after week 2.
4. **Build for yourselves first** — mentor agrees; if you two dogfood daily, keep building; don’t scale GTM until signal above.

**Tree viz:** Helpful for investigator / multi-path research (Twigg-like); not required for Agent DevOps wedge. Optional post-MVP polish.

**Naming (Q5):** Defer until “true MVP” — note only; ContextLayer working name fine.

### Enterprise path (mentor: devtools hard to sell)

**Category reframing:** Sell as **AI change governance / compliance evidence**, not “another devtool in the IDE.” Budget often comes from **security, platform eng, or GRC** — not individual developer credit cards.

| Stage | Buyer | What they pay for |
|-------|-------|-------------------|
| **Team** | Eng lead / EM | PR reasoning standard, fewer review cycles, shared hygiene pattern |
| **Mid-market** | Platform + security | Trace CI, redaction, “reasoning artifact required on AI-touched PRs” |
| **Enterprise** | CISO / compliance + eng enablement | Self-host, SSO, audit export, policy-as-code (`.gitllm/rules.yml` lane), evidence pack for regulators |

**Realistic yes, but not year-one default:** Enterprise buy is plausible **if** raw trace + PR export + CI gate prove audit value — same playbook as Snyk (dev surface → security budget) or Vanta-adjacent “prove process” buyers. **Without** trace CI and policy hooks, you stay prosumer/team tool.

**Mentor’s devtool warning still applies** to bottom-up IDE-only pitch. **Mitigation:** Lead with reviewer time + compliance story in outbound; keep desktop/MCP as delivery, not the category label.

---

## 7. Review queue (before implementation)

| Item | Owner | Status |
|------|-------|--------|
| Co-founder merge thesis (GitLLM × ContextLayer) | Both | **Reviewed — see §8** |
| Align on merged MVP scope + naming | Both | Pending — see [PRD-addendum-merged-vision.md](./PRD-addendum-merged-vision.md) |
| B1 PR export — UX + markdown template spec | — | **Spec + implementation — see [B1-PR-EXPORT-SPEC.md](./B1-PR-EXPORT-SPEC.md)** |
| B2 git-native path in dogfood | — | Not started |
| Graphify / Linear steal-list → concrete tickets | — | Captured above |
| GCC / Twigg / OneContext steals (§9) | Miles | **Captured — prioritize in §9.6** |
| Evaluation paper (§10) | Miles + agent | **Scoped — not started** |
| Cursor skill packaging (§9.5) | — | Design done; build after G1+T3 |
| OneContext import spike (§9.4) | — | Not started |
| Hosted/shared workspace | — | Deferred |

---

## 8. Co-founder merge — GitLLM thesis × ContextLayer (Jun 2026)

**Source:** Co-founder brainstorm + GPT iteration (pasted for team review). Originally a **separate idea**; team conclusion: **combine** into one platform rather than two products.

**Working names (TBD):** GitLLM, LLM Git, TraceForge, or keep **ContextLayer** as product with GitLLM as subsystem.

### 8.1 Partner thesis (summary)

| Layer | Idea |
|-------|------|
| **Problem** | AI multiplies PR volume; reviewers see diffs, not the **path** that produced them. Context is trapped in 1:1 dev↔agent chats. |
| **Vision** | Git-like versioning for **AI-assisted work**: branch conversation tangents, squash/clean before PR, link trace from GitHub PR. |
| **Collaboration** | Fork/join conversations; multiple accounts/agents contribute; eventually “open source conversations.” |
| **Enterprise** | Private repos, self-host, policy-as-code (`.gitllm/rules.yml`), trace CI (secrets, formatting, compliance), redaction — **GitHub playbook applied to traces**. |
| **Business** | Freemium → team → enterprise self-hosted (GitLab-style). |
| **MVP (GPT-refined)** | GitHub app: attach **policy-compliant AI development trace** to every PR — not “every prompt = commit.” |

**Key GPT corrections partner should adopt:**

- **Not** every prompt = commit → **checkpoint commits** (decision moments) + message-level replay internally.
- **Not** “Git for chats” consumer pitch → **AI provenance / governance for software teams**.
- **Not** hidden chain-of-thought → **observable engineering evidence** (prompts, tool calls, files read, tests, diffs, human edits).
- **Not** invent trace format alone → build on **[Agent Trace](https://github.com/cursor/agent-trace)** (format) + platform layer (GitHub-style).
- **Sequence:** PR trace → branch viewer → reviewer import context → team governance → public conversation repos (last).

### 8.2 What already exists (novelty check)

| Primitive | Exists? | Examples |
|-----------|---------|----------|
| Chat branching | Yes | ChatGPT branches, ContextBranch research |
| Prompt versioning | Yes | Langfuse, PromptLayer |
| Agent/LLM traces | Yes | LangSmith, Langfuse |
| AI PR review | Yes | GitHub Copilot Code Review |
| AI code provenance standard | Emerging | Agent Trace RFC |
| Structured reasoning graph (H/A/E/C) | **ContextLayer** | Hygiene, belief states — not raw chat |

**Novel combo (if merged well):** checkpointed traces + **epistemic structure** + PR handoff + governance CI + reviewer agent import.

### 8.3 How ContextLayer maps to GitLLM (merge model)

**Thesis:** ContextLayer is not competing with GitLLM — it is the **clean / squashed reasoning layer** inside the larger platform.

```
┌─────────────────────────────────────────────────────────────┐
│  AI session (Cursor, Claude Code, …)                        │
│       ↓ capture (partner: Agent Trace / raw observable)      │
│  Raw trace — audit, local, redacted (GitLLM recorder)       │
│       ↓ curate (squash, checkpoint, or MCP log)             │
│  ContextLayer workspace — blocks, H/A/E/C, hygiene          │
│       ↓ export (B1)                                         │
│  PR artifact + GitHub link — reviewer + trace CI            │
└─────────────────────────────────────────────────────────────┘
```

| GitLLM (partner lane) | ContextLayer (your lane) |
|------------------------|---------------------------|
| Capture prompts, tool calls, file reads, tests | Structure **why** into hypothesis / action / evidence / conclusion |
| Git branch = conversation tangent | Workspace or block-level “tangents”; belief state per path |
| Squash messy session → clean history | **Already:** timeline + health; **B1:** multi-block PR export |
| `.gitllm/rules.yml`, trace CI | Templates, admission rules, hygiene as **reasoning CI** |
| GitHub App, PR comment link | Markdown artifact content + deep link to hosted/shared workspace (later) |
| Self-host, SSO, redaction | Local-first today; enterprise deployment shared infra |
| Agent Trace file format | Compile **to** markdown + optional Agent Trace export |

**One-liner for merged pitch:**

> **Git tracks what changed. We track the auditable, reviewable reasoning path that produced the change — raw trace for compliance, structured graph for humans.**

### 8.4 Agreement points (both sides)

- Dual pitch stands: **investigator** (security/SWE) + **agent DevOps** (PR provenance).
- **Clean work tree** = partner’s squash + your structured blocks (same metaphor as git rebase before PR).
- **Information loss** between dev↔agent and team is the core pain — both ideas attack it.
- **Privacy:** private-first, self-host, `.gitllmignore` / redaction — not public conversation platform first.
- **Do not** build full “GitHub for everything” in v1.

### 8.5 Tension points (resolve before building)

| Topic | Partner bias | ContextLayer bias | Resolution direction |
|-------|--------------|-------------------|----------------------|
| Unit of history | Prompt/commit | Reasoning block | Checkpoints in raw trace; **blocks** in curated layer |
| Storage | Git repos of conversations | SQLite graph | Agent Trace/git for audit blobs; SQLite (or sync) for structured workspace — linked by PR ID |
| Capture | Auto from all AI tools | MCP + manual | Phase 1: Cursor MCP + optional trace importer; expand integrations later |
| Meeting/RFC repos | Broad platform | Narrow: investigations → code | Stage 5; start with **PR-linked technical decisions** only |
| Merge conversations | Semantic merge of branches | Block links + belief updates | Merge **insights/decisions**, not raw transcripts |

### 8.6 Merged MVP proposal (single wedge)

**Ship one thing:**

> **PR reasoning package:** GitHub PR + linked ContextLayer export (selected blocks) + optional raw Agent Trace attachment + pass/fail trace CI stub.

**Workflow:**

1. Dev works in Cursor; MCP logs to ContextLayer workspace (PR-anchored).
2. Dev selects blocks for this PR → export markdown.
3. (Partner) Recorder captures observable trace; dev squashes to checkpoints.
4. Open PR → bot comments summary + links + CI checks (secrets redacted, trace present, goal stated).
5. Reviewer reads structured export; optionally pulls context into own agent.

**What you have today vs gap:**

| Capability | Today | Gap |
|------------|-------|-----|
| Structured reasoning | ✅ | — |
| Hygiene / open loops | ✅ | — |
| MCP logging | ✅ | — |
| Multi-block PR export | ✅ | B1 shipped v1.2 |
| PR ↔ workspace link | ❌ | B2 metadata |
| Raw trace capture | ❌ | Partner / Agent Trace integration |
| GitHub App | ❌ | Partner |
| Trace CI / rules.yml | ❌ | Partner + hygiene rules export |
| Hosted shared workspace | ❌ | Later |

### 8.7 What NOT to merge into v1

- Public “open source conversations” social network
- Full meeting replacement / weekly standup repos
- Every-prompt Git history
- Competing with Langfuse/LangSmith on LLM app observability
- Replacing Copilot Code Review

### 8.8 Decisions & notes (solo execution phase)

**Execution model:** Co-founder in research fellowship — Miles executes **both lanes sequentially**, not “finish reasoning lane and wait.” After reasoning MVP (B1 export + dogfood), Miles continues into capture/platform (checkpoints, redaction, GitHub automation, trace CI) while co-founder reviews async and rejoins when fellowship ends.

**Speed:** Category is heating up (Agent Trace RFC, Graphify, Copilot review). Ship wedge fast; expand into co-founder’s lane without blocking on his availability.

**Checkpoints alignment:** Yes — GPT thread explicitly rejected “every prompt = commit.” Use **checkpoint commits** in raw trace + **blocks** in curated ContextLayer layer. Partner GPT convo and this doc agree.

**Agent Trace (2026-06-07):** Interoperate optionally; **do not** subsume product. Agent Trace = attribution format only (line → conversation URL). Our moat = structured reasoning, hygiene, PR narrative, governance. See [POSITIONING.md](./POSITIONING.md).

**Integrations:** ContextLayer curated layer is **LLM-agnostic** (desktop + MCP). Auto raw capture varies by tool — target Cursor + Claude Code + Codex + GitHub over time; MCP first.

**v1 “viewer”:** Author uses **desktop app** while working; **PR export markdown** is what reviewers read on GitHub. Web reviewer viewer = later.

**Copy:** Canonical one-liner in [POSITIONING.md](./POSITIONING.md).

### 8.9 Open questions (when co-founder returns)

1. Final product name
2. Who owns GitHub App + self-host vs reasoning app long-term
3. Whether to emit Agent Trace records in v1 or v2

### 8.10 Recommended next doc (together)

- ~~**Merged PRD addendum**~~ → **[PRD-addendum-merged-vision.md](./PRD-addendum-merged-vision.md)** (draft for co-founder sign-off)
- **Pitch:** [PITCH.md](./PITCH.md)
- **Do not** rewrite locked ContextLayerPRD v1.0 until merge scope is signed off — use addenda until PRD v2.0

---

## Changelog

| Date | Change |
|------|--------|
| 2026-06-07 | Initial doc: dual pitch, B1 export, B2 collaboration sequencing, Graphify/Linear inspo, B4 storage decision |
| 2026-06-07 | §8: Co-founder GitLLM merge analysis and merged MVP proposal |
| 2026-06-07 | Solo execution: both lanes sequential; POSITIONING.md added |
| 2026-06-07 | §9–§11: GCC/Twigg/OneContext steals, eval paper scope, skill+SQLite architecture, mentor ICP notes |
| 2026-06-07 | G1 scheduled; §9.4/§9.5 clarifications; [EVAL-PAPER.md](./EVAL-PAPER.md); enterprise path §11 |
| 2026-06-07 | G2 revised: defer checkpoint table to capture lane; O1 + capture reprioritized in §9.6 |
| 2026-06-07 | **T2 fork block UX dropped** from active backlog (block links + capture branch) |

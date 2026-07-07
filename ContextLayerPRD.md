# ContextLayer — Product Requirements Document

**Subtitle:** Reasoning Graph System  
**Version:** 1.0 (Reasoning Graph) — **locked**  
**Last updated:** 2026-06-12  
**Status:** Locked — Phase 1 build

> **Current scope:** Parent v1.0 invariants still apply. For **blocks, MCP, PR export, and merged co-founder vision**, see addenda:
> - [docs/PRD-addendum-blocks.md](./docs/PRD-addendum-blocks.md)
> - [docs/PRD-addendum-merged-vision.md](./docs/PRD-addendum-merged-vision.md)
> - [docs/POSITIONING.md](./docs/POSITIONING.md)

> **Deprecated:** v0.1 (cross-AI context compiler, GPT→Cursor wedge) is retired. See git history / prior commits for legacy spec. Cross-AI continuity moves to Phase 4.

---

## 0. One-liner

A **persistent reasoning graph** for long-running work — workspaces, hypotheses, tests, evidence, and conclusions — where AI tools are interfaces into one system of record; **cross-AI continuity comes after the graph proves valuable**.

---

## 1. Product vision

### What this is

ContextLayer is a **bounded epistemic tracking system** for human investigative work — a structured external reasoning system that standardizes how understanding evolves over time. It tracks not what the user said, but **how beliefs change, get tested, and get invalidated**.

> **Design law:** You are not building a compilable platform. You are building a **constrained reasoning state machine** that can be compiled later.

The system preserves **epistemic state changes** — how knowledge formed, failed, and evolved — not static knowledge.

### What this is NOT

- ❌ AI memory / chat history storage
- ❌ Semantic note retrieval (Obsidian + embeddings)
- ❌ Generic knowledge base or personal assistant memory
- ❌ General computation graph or universal knowledge engine
- ❌ Compile framework or “platform for all compile systems” (compile is a future view, not the product)
- ❌ Bug bounty / pentest tool (security is dogfood wedge, not ceiling)
- ❌ Cross-AI sync product (Phase 4 only)

### Core insight

Across domains (security research, software, startups, legal, investing), the same cognitive loop repeats:

**Goal → Hypotheses → Tests/Actions → Evidence → Conclusions → Iteration**

The product models this loop explicitly as a **directed reasoning graph over time**, not a flat note store.

### Strategic framing

> **Git for thinking** — workspace = repo, hypothesis = branch, action = commit attempt, evidence = diff, conclusion = merge/close, failed ideas = closed branches.

Cross-AI sync is not the invention; it is the GitHub Desktop / VS Code / CLI layer on top — useful after the core is proven.

### AI role

AI is **not** the system of record. AI is an **annotator** that may suggest hypotheses, links, and summaries. All structured state is **deterministic and user-owned**. AI outputs require explicit user confirmation before becoming graph state.

---

## 2. Phase roadmap

| Phase | Focus | Includes |
|-------|--------|----------|
| **1 (NOW)** | Manual reasoning graph | CRUD, timeline UI, admission rules, copy summary, local SQLite |
| **1.5** | LLM workspace summary | Only after Phase 1 success metrics hit — suggest/classify, never auto-commit |
| **2** | AI augmentation | Hypothesis suggestions, contradiction detection, similar-past retrieval |
| **3** | Ingestion tools | Manual import/export, paste chat → suggest nodes, CLI helpers |
| **4** | Cross-AI continuity | Same workspace in GPT/Cursor/Claude; `compile_workspace(id, intent)` → injection packet |

**Phase 1 has zero integrations:** no MCP, no Custom GPT Action, no ingest API, no browser extension.

---

## 3. Phase 1 MVP scope

### MUST include

1. Create workspace (name, **required goal**, optional template)
2. Add hypothesis, action, evidence, conclusion (any order — loose entry)
3. Link nodes (many-to-many in data model; simple link UI)
4. Mark conclusions with outcome + optional tag + confidence
5. View workspace as **timeline** (graph structure underneath, no graph viz)
6. Correction events on edit (append-only history)
7. Soft-delete nodes
8. **Copy workspace summary** — markdown dump of linked reasoning state (no AI)
9. Filter timeline by node type
10. Workspace templates: Blank / Security hunt / Product research (placeholders only)

### MUST NOT include (Phase 1)

- Cross-AI integrations (GPT, Cursor, Claude)
- MCP server, ingest API, Custom GPT
- Browser extension, auto-sync
- Embeddings / semantic search
- Graph visualization
- Image/file attachments
- Mobile, web, multi-user auth
- AI-generated graph writes
- Cross-workspace linking

### Three screens (desktop only)

1. **Workspace list** — create, pick template
2. **Workspace timeline** — chronological nodes, type filter, unlinked badges
3. **Node creation / detail panel** — add/edit/link nodes

### Stack

- **Tauri 2 + React + Tailwind** — desktop shell
- **Rust sidecar** — SQLite via sqlx/rusqlite
- **Local-first** — `~/.contextlayer/graph.db`
- No web architecture in Phase 1

### Dogfood (two workspaces, day one)

1. **ContextLayer product** — MVP validation, product hypotheses
2. **Security domain** — e.g. LLM pentest suite evaluation or hunt workspace

Do not combine into one hybrid workspace — cognitive separation is intentional.

---

## 4. V1 enforcement spec (data admission + state integrity)

> The system does not constrain thinking. It only constrains **what counts as a valid change in belief state**. Only structured reasoning transitions are first-class data.

### 4.0 Core principle

Nothing enters the system unless it represents a step in reasoning. If it does not change, test, support, or invalidate thinking — it does not exist as a stored entity.

**No pure memory test:** If content would be useful even if you never run another test or action → reject or force conversion.

**Thinking graph test:** If a node cannot connect to hypothesis → test → evidence chain (eventually) → it does not belong in clean export.

---

### 4.1 Allowed data types (strict schema)

Only these entities may exist. **No fifth type. No "notes" table.**

#### Explicit exclusion: Constraint is not first-class

**Constraint is not a stored entity** and must never become a parallel primitive. Represent constraints only as:

| Intent | Map to |
|--------|--------|
| Testable rule or boundary | Hypothesis |
| Observed restriction from reality | Evidence |
| Decision about scope or approach | Conclusion tag (`pivot`, `act`, `ignore`, `defer`) |

Do not add a `constraints` table. Do not reserve Constraint for a future schema version.

#### Workspace

| Field | Required |
|-------|----------|
| id | yes |
| name | yes |
| goal | yes (string) |
| template | optional enum: `blank \| security_hunt \| product_research` |
| created_at, updated_at | yes |

All nodes MUST belong to a workspace. No global nodes. No cross-workspace links in Phase 1.

#### Hypothesis

A falsifiable or uncertain claim.

| Field | Required |
|-------|----------|
| id, workspace_id | yes |
| text | yes |
| status | `open \| testing \| rejected \| supported` (derived from conclusions where applicable; `open` default) |
| created_at | yes |
| deleted_at | soft-delete |

**Rules:**
- Must be a claim that can be tested or invalidated
- May exist without links (initial/draft state)
- Rejected hypotheses remain **visible** — failed thinking is product value

#### Action (Test)

An operation performed to evaluate a hypothesis.

| Field | Required |
|-------|----------|
| id, workspace_id | yes |
| text | yes |
| created_at | yes |
| deleted_at | optional |

**Rules:**
- Represents something **done**, not observed knowledge
- `linked_hypothesis_ids` — optional at create (many-to-many via `node_links`)

#### Evidence

Raw observed output from reality.

| Field | Required |
|-------|----------|
| id, workspace_id | yes |
| text | yes |
| source | optional URL |
| created_at | yes |
| deleted_at | optional |

**Rules:**
- Must be **observable**, not interpreted (interpretation → Conclusion)
- `linked_action_ids` — optional at create (many-to-many)

#### Conclusion (state transition — strictly validated)

The only entity with **hard admission constraints**.

| Field | Required |
|-------|----------|
| id, workspace_id | yes |
| text | yes (interpretation) |
| outcome | `confirmed \| rejected \| uncertain \| refined` |
| tag | optional: `none \| pivot \| act \| ignore \| defer` |
| confidence | optional 0–1 or low/med/high |
| created_at | yes |
| superseded_at | optional |

**HARD CONSTRAINT — block save if violated:**
- ≥1 linked hypothesis (`linked_hypothesis_ids`)
- ≥1 linked evidence (`linked_evidence_ids`)

Error message: *"Conclusion requires at least one hypothesis and one evidence item."*

**Rules:**
- Multiple conclusions per hypothesis allowed (append-only)
- Latest non-superseded conclusion is **derived**, not enforced by overwrite
- Represents state change, not journaling

---

### 4.2 Admission rules (data firewall)

#### Classification rule

All input must map to: Hypothesis | Action | Evidence | Conclusion.

If it does not → reject OR force conversion (e.g. "GraphQL often has auth issues" → hypothesis: "GraphQL APIs may have auth issues in this target").

#### Forbidden as standalone entities

- Generic notes, summaries, explanations
- Research dumps, learning notes
- TODO lists / checklists (unless attached to hypothesis or action)
- Raw chat transcripts (extract structured nodes in Phase 3+ only)

**Obsidian prevention rule:** If it can exist meaningfully outside a reasoning chain, it does not belong.

#### Orphan / unlinked rule (soft enforcement — Phase 1)

Nodes may exist unlinked temporarily BUT:
- Marked `unlinked` in UI
- Excluded from **clean workspace summary** export
- UI warning: *"This does not yet participate in the reasoning graph."*

Loose **order**, strict **type**. Links optional at create; conclusions require links at save.

---

### 4.3 Temporal model (append-only)

- Entities are immutable once created
- **Edit** = correction event + new version row (`entity_versions`)
- Old versions remain in history; UI shows current + expandable history
- Soft-delete only — nothing silently hard-deleted
- All mutations append to `events` audit log

---

### 4.4 AI behavior rules

**Phase 1:** Zero AI graph writes.

**Phase 1.5+:** AI may suggest hypotheses, links, summaries. AI may NOT:
- Create or commit nodes without user confirmation
- Inject "facts" as nodes
- Bypass validation rules
- Create summary/note entity types

---

### 4.5 Clean state export rule

**Copy workspace summary** includes only:
- Linked hypotheses, actions, evidence
- Valid conclusions (meeting hard constraints)

Unlinked nodes: excluded OR listed under `"Unstructured (draft)"` section.

This export is a **manual escape hatch** and primitive `compile_workspace()` prototype — not an integration.

---

### 4.6 Failure mode protection

System is broken if:
- Users primarily store unlinked text (notes-with-labels)
- Conclusions exist without evidence chains
- Workspace devolves into flat list of entries

Kill or redesign if after 2 weeks the tool feels like overhead vs Notion/chat logs.

---

## 5. Data model (SQLite)

```
workspaces(id, name, goal, template, created_at, updated_at)

hypotheses(id, workspace_id, text, status, created_at, deleted_at)
actions(id, workspace_id, text, created_at, deleted_at)
evidence(id, workspace_id, text, source, created_at, deleted_at)
conclusions(id, workspace_id, text, outcome, tag, confidence, created_at, superseded_at)

node_links(id, workspace_id, from_type, from_id, to_type, to_id, created_at)
  -- many-to-many: hypothesis↔action, action↔evidence, conclusion↔hypothesis, conclusion↔evidence

entity_versions(id, entity_type, entity_id, version, body_json, created_at)
events(id, type, entity_type, entity_id, payload_json, created_at)
  -- types: created, corrected, soft_deleted, link_added, link_removed
```

**Indexes:** workspace_id on all node tables; created_at for timeline ordering; node_links by from/to.

---

## 6. UX behaviors

### Timeline

- Sort: `created_at` descending default; toggle ascending
- Filter: by node type (hypothesis / action / evidence / conclusion)
- Visual: type badge, unlinked warning badge, rejected/superseded styling

### Node creation

- **Pick type first** — no blank-page note mode
- Placeholders per template (security vs product)
- Link picker: optional on create, required for conclusion save

### Edit / delete

- Edit → correction event + version bump
- Delete → soft-delete + event log entry

---

## 7. Success criteria (Phase 1, ~2 weeks)

**Pass (need both):**
1. Return to a workspace after 24–72h and immediately understand context without re-reading old chats
2. Successfully trace a conclusion back through hypothesis → evidence chain ≥5 times in real use

**Optional third:** Stop using Notion/chat logs for the same work entirely.

**Fail:** System feels like homework; revert to notes; no retrieval benefit.

---

## 8. Week 1 ship order

| Day | Deliverable |
|-----|-------------|
| 1 | SQLite schema + migrations + events log |
| 2 | CRUD + linking API (Tauri commands or local REST) |
| 3 | Workspace list + timeline screens |
| 4 | Node panel + correction/soft-delete + conclusion validation |
| 5 | Copy workspace summary + create two dogfood workspaces |
| 6–7 | **Use only** — log friction, no new features |

---

## 9. Phase 4 preview (deferred — legacy v0.1 concepts live here)

When reasoning graph is proven:

- **Ingestion (Phase 3):** import chat → suggest nodes (user confirms each)
- **Continuity (Phase 4):** MCP / Actions inject `compile_workspace(workspace_id, intent)` packet into GPT, Cursor, Claude
- Reframed compiler: not "pull PRD" — **compile linked reasoning state** for prompt injection
- Local-first + append-only event log concepts from v0.1 carry forward

---

## 10. Global design law

> The system does not store knowledge.  
> It stores validated transitions in understanding.

---

## 11. Repo layout (Phase 1)

```
ContextLayer/
├── ContextLayerPRD.md
├── apps/desktop/              # Tauri 2 + React
├── crates/
│   ├── core/                  # domain + validation (admission rules)
│   ├── db/                    # SQLite migrations + queries
│   └── export/                # workspace summary markdown
├── migrations/
├── fixtures/workspaces/       # sample graph states for tests
└── .taskmaster/
```

---

## 12. Test strategy

- **Validation tests:** conclusion blocked without hypothesis + evidence links
- **Admission tests:** no note entity type in schema
- **Export tests:** unlinked nodes excluded from clean summary
- **Correction tests:** edit creates version + event, prior text preserved
- **Integration:** create workspace → add chain → export → assert markdown structure

---

## 13. Compilation interface (future system contract)

> **Not in Phase 1 MVP.** Constraint contract only — not a feature design, not platform expansion. ContextLayer remains a reasoning state machine; compile is a **view** over it.

The system must support **deterministic export** of workspace state into read-only structured formats.

### Required properties

| Property | Requirement |
|----------|-------------|
| Input | Workspace graph only (core primitives + links + versions) |
| Output | Structured representation of reasoning chains |
| Mutation | No mutation of source graph |
| Recompute | Recomputable at any time from stored state |
| Metadata | Must not require fields outside core primitives |

### Allowed input types (at rest)

- Hypothesis
- Action
- Evidence
- Conclusion

No sixth stored type without an explicit schema revision. **Constraint is not eligible** — see §4.1. Domain-specific concepts (e.g. citation verification scores) must map to Hypothesis, Action, Evidence, or Conclusion — not parallel tables.

### Critical constraint (anti-rot)

> **Compiled outputs must be derivable without introducing new state or requiring any fields not present in the core schema.**

Without this rule, implementations drift toward:

- Report-only fields stored separately from the graph
- Hidden scoring or metadata layers for domain convenience
- **ContextLayer + parallel reporting schema** (system rot)

Compile functions are **pure views**: `compile(workspace_id, profile) → read_only_output`. They do not create, update, or depend on shadow state.

### Domain compile profiles

Domain products may ship **compile profiles** — one-way renderers from graph → read-only output (e.g. CiteLock audit report). Profiles must not introduce domain tables in the core store or expand ContextLayer into a generic compile SDK.

Phase 1 **Copy workspace summary** is the minimal compile prototype.

---

### 13.1 CiteLock as derived layer (non-binding)

[CiteLock](../CiteLock/docs/PRD-citelock-v1.md) (research source layer: capture, archive, verify citations) can eventually sit **on top of** ContextLayer rather than as a separate reasoning system.

| CiteLock concept | ContextLayer mapping |
|------------------|----------------------|
| Research project | Workspace (`goal`: verifiable submission) |
| “Is this citation real?” | Hypothesis |
| CrossRef / clip / URL check | Action |
| API response, archive snapshot | Evidence |
| verified / warning / failed | Conclusion |
| Audit report / `VerificationRun` export | Compile output (derived view) |

CiteLock PRD §21 already follows this pattern: exporters are read-only renderers of immutable runs, never re-verify. ContextLayer generalizes that contract to all compile paths.

**Relationship to build order:**

- ContextLayer Phase 1 must be dogfooded and validated first.
- CiteLock v1 may continue as a separate codebase; this section is a **contract**, not a commitment to merge repos in MVP.
- If ContextLayer proves valuable, new CiteLock work should **write reasoning into the graph** and treat library/audit UI as compile views — not grow a second ontology.

---

## Appendix A — Legacy v0.1 (retired)

Cross-AI context compiler, CONTEXT JSON ingest, Custom GPT Action, MCP `pull_project_context`, project/artifact/decision model — **deferred to Phase 3–4**. Do not implement in Phase 1.

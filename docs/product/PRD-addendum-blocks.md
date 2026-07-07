# ContextLayer PRD Addendum — Blocks, Belief States & Epistemic Tags

**Version:** 1.2  
**Status:** Phase 1.2 in progress  
**Parent:** [ContextLayerPRD.md](../ContextLayerPRD.md) v1.0  
**Last updated:** 2026-06-13  
**Related:** [FUTURE-IMPLEMENTATION.md](./FUTURE-IMPLEMENTATION.md) (GTM / post-demo backlog — not locked spec)

---

## 0. Why this addendum exists

Dogfood feedback: separate timeline rows per node type become overwhelming during real investigations. The product must feel like a **reasoning OS**, not "Obsidian with stricter forms."

**Design law (unchanged):** Four typed primitives remain in storage. The **Block** is the primary UX unit — one timeline row containing zero-to-four fields, auto-linked internally.

**Differentiation law:** ContextLayer wins by surfacing **epistemic state** and **reasoning debt** — things Obsidian cannot know without typed graph structure.

---

## 1. Block (primary UX unit)

### Definition

A **Block** is one reasoning step in a workspace timeline. It may contain any subset of:

| Field | Primitive | Required for save? |
|-------|-----------|------------------|
| Hypothesis | hypothesis | No |
| Action | action | No |
| Evidence | evidence | No |
| Conclusion | conclusion | No |

**Save rule:** At least one field must be non-empty.

**Auto-linking (within block):** When multiple fields exist:
- hypothesis → action (if both)
- action → evidence (if both)
- conclusion → hypothesis + evidence in same block (if conclusion present)

Text stored **verbatim** — no normalization.

### Block metadata

| Field | Purpose |
|-------|---------|
| `belief_state` | Epistemic state (see §2) |
| `system_tag` | Structured signal (see §3) |
| `user_tag` | Free-text custom label |

### Block-to-block links

Many-to-many within workspace. Cross-workspace links not allowed.

### Legacy nodes

Pre-1.1 nodes migrate to single-field blocks on upgrade.

---

## 2. Belief states

Manual, user-controlled:

| State | Meaning |
|-------|---------|
| `open` | Not yet evaluated |
| `leaning_true` | Evidence trending toward confirmation |
| `leaning_false` | Evidence trending toward rejection |
| `confirmed` | Accepted as true |
| `rejected` | Ruled out |

**Belief state history:** Append-only `belief_state_history` per block.

---

## 3. Tag system (hybrid)

### System tags (block)

| Tag | When to use |
|-----|-------------|
| `none` | Default |
| `needs_review` | Contradiction or uncertainty |
| `ruled_out` | Dead end / disproven |
| `reportable` | Security: worth filing |
| `reasoning_debt` | Missing conclusion/evidence; open too long |
| `stale` | No activity in N days (manual or auto-suggest later) |

### User tags

Free-text optional label per block.

### Conclusion decision tags

`pivot` / `act` / `ignore` / `defer` — powers Decision Log view.

### Confidence (conclusions)

**Low / Medium / High** (not percentages in UI).

---

## 4. Phase roadmap

### Phase 1.1 — Blocks ✅ DONE

- [x] Block CRUD + timeline UI
- [x] Intra-block auto-link + block-to-block links
- [x] Belief states + history table
- [x] System + user tags; confidence levels
- [x] MCP `save_block` + block-aware export

### Phase 1.2 — Reasoning hygiene (NOW)

Schema: **no new tables** — aggregate queries over blocks.

| Feature | Status | Description |
|---------|--------|-------------|
| **Workspace health panel** | ✅ | Summary counts: blocks, open, confirmed, rejected, needs_review, reasoning_debt |
| **Orphan detection** | ✅ | Action w/o evidence; evidence w/o hypothesis; hypothesis-only; incomplete conclusions |
| **Stale blocks** | ✅ | Open/leaning 14+ days with no evidence or conclusion |
| **Why is this still open?** | ✅ | Open hypothesis with no action taken |
| **Dead-end library** | ✅ | Rejected belief or `ruled_out` tag — avoid retesting |
| **Decision log** | ✅ | Conclusions tagged pivot / act / ignore / defer |
| **Hygiene → timeline filter** | ✅ | Click category to filter main timeline |
| **Blocked reason field** | DEFER | Optional text: "need endpoint access" — Phase 1.3 |
| **Auto-suggest stale/reasoning_debt tags** | DEFER | Heuristic tag suggestions — Phase 1.3 |

**Acceptance criteria (1.2):**
1. Health panel visible on every workspace timeline
2. Each hygiene category lists actionable blocks; click opens block editor
3. Counts match block data after save/delete
4. Stale threshold = 14 days (configurable later)

---

### Phase 1.3 — Contradiction & belief evolution (DEFER)

| Feature | Schema | Description |
|---------|--------|-------------|
| **Contradiction queue** | `evidence.polarity`: support / contradict / neutral | Surface blocks where supporting vs contradicting evidence diverges |
| **Belief change timeline** | Uses `belief_state_history` | Click block → chronological belief transitions |
| **What changed my mind?** | Event + link metadata | Conclusion shows evidence that drove it; overturn events when belief flips |
| **Confidence drift** | `confidence_level_history` table | Track Low → Medium → High changes after new evidence |
| **Evidence strength** | `evidence.strength`: weak / moderate / strong / verified | Distinguish anecdote vs measurement |
| **Confidence mismatch detection** | Heuristic | Flag High confidence + weak/single evidence |
| **Blocked reason field** | `blocks.blocked_reason` text | "Why is this still open?" user-entered blocker |
| **Auto-suggest system tags** | Heuristics | Suggest `stale`, `reasoning_debt`, `needs_review` |

---

### Phase 1.5 — LLM augmentation (DEFER, gated on dogfood)

Requires Phase 1 LLM summary gate from main PRD. **Never auto-commit.**

| Feature | Description |
|---------|-------------|
| **Next best question** | Given open blocks, suggest the test that most reduces uncertainty |
| **What would falsify this?** | Per open hypothesis — required falsification criteria |
| **What would change my mind?** | Explicit evidence that would flip belief state |
| **Belief state suggestions** | LLM proposes state change; user confirms |
| **Contradiction summarization** | LLM reads contradicting evidence pairs; user tags `needs_review` |
| **Assumption extraction** | LLM suggests untested assumptions from hypothesis text |

---

### Phase 2 — Reasoning OS moat (DEFER)

| Feature | Why Obsidian can't do it |
|---------|--------------------------|
| **Reasoning debt dashboard** | Cross-workspace aggregate of open/stale/untested |
| **Investigation replay** | Ordered block chain — "how we got here" |
| **First-class assumptions** | Primitive or tagged subtype; track tested vs untested |
| **Dead-end search** | Full-text + belief filter across workspaces |
| **Cross-workspace patterns** | Local stats: "you conclude early", "assumptions untested" |
| **Personal reasoning profile** | Failure modes + strengths from event log |

---

## 5. What ContextLayer can say that Obsidian cannot

Schema-derived (no AI required):

1. These N conclusions rest on the same untested hypothesis-only block.
2. You have 7 open blocks with no action in 30 days.
3. This conclusion is High confidence but supported by only 1 evidence block.
4. You already disproved a similar path (rejected / ruled_out) in this workspace.
5. 3 blocks tagged needs_review — possible contradiction.
6. Reasoning debt: 4 open hypotheses, 0 conclusions this week.
7. Orphan: action recorded with no evidence attached.

With Phase 1.3+:

8. Evidence E-12 contradicts E-08 on the same hypothesis.
9. Belief on block B went Open → Leaning True → Rejected over 6 days.
10. Across 12 workspaces, rejected hypotheses often lacked direct evidence.

---

## 6. Non-goals (still)

- Graph visualization (Phase 2+ maybe)
- Semantic / embedding search
- Auto-commit from AI
- Cross-workspace shared knowledge base
- Mobile / web / multi-user

---

## 7. Acceptance criteria (Phase 1.1) ✅

1. One block form → one timeline row with auto-links
2. Block-to-block links work
3. Belief history logged on state change
4. Legacy data migrates
5. MCP + desktop share DB

---

## 8. Acceptance criteria (Phase 1.2)

1. Workspace health panel shows live counts
2. Orphans, stale, still-open, dead-ends, decisions each list blocks
3. Click hygiene item → opens block in editor
4. Category selection filters timeline
5. Hygiene refreshes after block save/delete

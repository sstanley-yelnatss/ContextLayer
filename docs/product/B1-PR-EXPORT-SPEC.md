# B1 — PR reasoning export (Phase 1)

**Status:** Implementing  
**Scope:** Tonight demo + Phase 1 exit criteria  
**Non-goals:** CLI, GitHub App, committed-file automation (Phase 2)

---

## Problem

Reviewers need a **PR-sized reasoning receipt**, not the full workspace dump.

## User flow

1. Open workspace timeline.
2. Enable **PR export** mode (or use checkboxes on blocks).
3. Select one or more blocks that explain **this** change.
4. Click **Export for PR** → markdown copied to clipboard.
5. Paste into GitHub PR description (demo tonight) or commit file (Phase 2).

## Rust API (`crates/export`)

```rust
/// Full workspace — unchanged behavior.
compile_workspace_summary_markdown(store, workspace_id) -> String

/// Selected blocks only; chronological order (oldest `updated_at` first).
compile_pr_export_markdown(store, workspace_id, block_ids: &[String]) -> String
```

- Empty `block_ids` → error `"Select at least one block"`.
- Unknown block ID → error listing missing IDs.
- Block from another workspace → excluded / error if none match.
- No SQLite schema changes.

## Markdown template (PR export)

Plain, GitHub-readable — no UUIDs, no bold/italic noise:

```markdown
PR Reasoning: {workspace.name}

Goal: {goal}

Summary for reviewers:
{latest conclusion text from selected blocks, if any}

---

Belief: Open · Outcome: confirmed · Confidence: high

Hypothesis:
{text}

...

---

Exported from ContextLayer (N of M blocks in this workspace).
Hygiene: X of N exported block(s) still have unsettled belief; Y incomplete.
```

## Desktop UI

| Control | Behavior |
|---------|----------|
| **PR export** toggle | Shows checkboxes on timeline rows |
| Checkbox | Toggle selection; does not open block panel |
| **Select visible** | All blocks in current filtered view |
| **Clear** | Clear selection |
| **Export for PR** | Requires ≥1 block; copies PR markdown |
| **Copy summary** | Unchanged — full workspace |

## Tauri

`export_pr_reasoning(workspace_id, block_ids: Vec<String>) -> String`

## MCP

`export_blocks(workspace_id, block_ids: Vec<String>) -> markdown text`

## Acceptance criteria

- [ ] Select 1+ blocks → PR markdown only includes those blocks.
- [ ] Blocks appear in chronological order (oldest first).
- [ ] Full **Copy summary** still exports all blocks.
- [ ] Invalid/empty selection returns clear error.
- [ ] Unit tests in `crates/export`.

## Phase 2+ (not B1)

See [FUTURE-IMPLEMENTATION.md](./FUTURE-IMPLEMENTATION.md) §1.4 — PR handoff & discoverability.

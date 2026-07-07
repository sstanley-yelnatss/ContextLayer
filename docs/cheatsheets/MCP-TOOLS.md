# ContextLayer MCP — Tool Reference

Quick sheet for all tools exposed by `contextlayer-mcp`. Same database as the desktop app: `~/.contextlayer/graph.db`.

**Command cheat sheet (CLI + capture flow):** [COMMANDS-CHEATSHEET.md](./COMMANDS-CHEATSHEET.md)

**Agent rules (built into server):**
- Store text **verbatim** — do not rewrite user wording.
- Only write when the user asks you to log something.
- **Tiered reads:** `get_workspace_index` first → `get_block` for details → `get_workspace_hygiene` before next tests.
- **Prefer `save_block`** over legacy `create_*` tools.

### Context tiers

| Tier | Tool | Returns |
|------|------|---------|
| 0 | `list_workspaces` | All workspace ids + goals |
| 1 | `get_workspace_index` | Block titles, belief, hygiene flags — **no body text** |
| 2 | `get_block` | Full single block |
| 3 | `export_blocks` / `compile_agent_context` | Compiled markdown slice |

---

## Workspaces

| Tool | What it does | Key params |
|------|----------------|------------|
| **`list_workspaces`** | List all workspaces (id, name, goal, template). Call first if workspace is unknown. | — |
| **`create_workspace`** | Create a new workspace. | `name`, `goal`, `template` (`blank` \| `agent_devops` \| `security_hunt` \| `product_research` \| `decision_strategy`; default `agent_devops`) |
| **`update_workspace`** | Rename or change goal/template. Omitted fields stay unchanged. | `workspace_id`, optional `name`, `goal`, `template` |
| **`delete_workspace`** | **Permanent** delete — workspace + all blocks, links, data. Irreversible. | `workspace_id` |

---

## Read / export

| Tool | What it does | Key params |
|------|----------------|------------|
| **`get_workspace_index`** | **Tier-1 index** — goal, hygiene summary, per-block id/title/belief/hygiene flags. **No body text.** Prefer over `get_workspace_summary` for scans. | `workspace_id` |
| **`get_workspace_summary`** | Full workspace dump as markdown — **all** block bodies. High token cost; use for one-off full export. | `workspace_id` |
| **`get_workspace_hygiene`** | Reasoning health report — orphans, stale paths, open loops, dead ends, decision log. JSON. | `workspace_id` |
| **`list_blocks`** | Lightweight block index: id, title, belief_state, incomplete flag. | `workspace_id` |
| **`get_block`** | One block with full hypothesis / action / evidence / conclusion fields. | `workspace_id`, `block_id` **or** `block_title` |
| **`compile_agent_context`** | **Agent packet** — full bodies + block IDs + hygiene snapshot. Omit `block_ids`/`block_titles` for entire workspace. | `workspace_id`, optional `block_ids`, `block_titles` |
| **`export_blocks`** | **PR-ready markdown** for a **selected subset** of blocks (same as desktop “Export for PR”). Paste into GitHub PR description. | `workspace_id`, `block_ids` and/or `block_titles` (at least one required) |
| **`import_session`** | Paste Cursor/chat transcript → new workspace with draft blocks (`needs_review`). Verify in desktop. | `workspace_name`, `transcript`, optional `goal` |

---

## Capture — opt-in session log

Live capture **does not run** until `start_capture` (MCP) or `contextlayer-recorder start` (CLI). Storage: `~/.contextlayer/capture/{workspace_id}/` (`log.jsonl` + `commits.jsonl`). Full flow: [COMMANDS-CHEATSHEET.md](./COMMANDS-CHEATSHEET.md).

| Tool | What it does | Key params |
|------|----------------|------------|
| **`bind_capture_project`** | Map Cursor project → workspace (no recording). | `workspace_id`, `cursor_project`, optional `repo_path` |
| **`start_capture`** | Open capture gate; baseline existing transcript bytes. | `workspace` (UUID or **name**), optional `cursor_project`, `transcript_path`, `label` |
| **`stop_capture`** | Close capture gate. | `workspace` (UUID or name) |
| **`capture_status`** | List active capture sessions. | — |
| **`get_context_log`** | Windowed session log (user/assistant turns). | `workspace_id`, optional `from_seq`, `limit` |
| **`get_context_commits`** | Recent decision commits with log seq ranges. | `workspace_id`, optional `limit` |
| **`commit_checkpoint`** | Decision checkpoint — slices log seq range. Fails if log empty. | `workspace_id`, `intent`, `note`, optional `rejected_paths`, `git_sha`, `block_ids` |
| **`list_checkpoints`** | List checkpoints for workspace. | `workspace_id` |
| **`get_trace_summary`** | Message + checkpoint counts. | `workspace_id` |
| **`append_trace_event`** | Append trace metadata event (legacy compat). | `workspace_id`, `event_type`, `summary`, optional `payload` |
| **`import_session`** | Paste transcript → new workspace + draft blocks + log. | `workspace_name`, `transcript`, optional `goal` |

Trace CI (repo): `.contextlayer/rules.yml` + `cargo run -p contextlayer-trace-cli -- check`.

### `export_blocks` output includes
- Workspace name + goal
- Summary for reviewers (from newest conclusion in selection)
- Each block: belief, outcome, confidence, Hypothesis / Action / Evidence / Conclusion
- Footer: N of M blocks exported
- Hygiene note if any exported blocks have unsettled belief or are incomplete

Blocks ordered by **created_at** (oldest first).

---

## Write — blocks (preferred)

| Tool | What it does | Key params |
|------|----------------|------------|
| **`save_block`** | Create or update one timeline block. Title-only create is OK. On update, **only send fields you want to change** — rest are preserved. | `workspace_id`, optional `block_id` or `block_title`, optional `title`, `hypothesis_text`, `action_text`, `evidence_text`, `evidence_source`, `conclusion_text`, `conclusion_outcome`, `conclusion_tag`, `confidence_level` (`low` \| `medium` \| `high`), `belief_state`, `system_tag`, `user_tag`, `link_to_block_ids` |
| **`delete_block`** | Soft-delete a block. | `workspace_id`, `block_id` **or** `block_title` |

**Belief states:** `open` \| `leaning_true` \| `leaning_false` \| `confirmed` \| `rejected`  
**System tags:** `none` \| `needs_review` \| `ruled_out` \| `reportable` \| `reasoning_debt` \| `stale`  
**Conclusion outcome:** `confirmed` \| `rejected` \| `uncertain` \| `refined`  
**Conclusion tag:** `none` \| `pivot` \| `act` \| `ignore` \| `defer`

---

## Write — legacy nodes (avoid if possible)

Lower-level tools from pre-block model. Use **`save_block`** instead unless you need loose unlinked nodes.

| Tool | What it does | Key params |
|------|----------------|------------|
| **`create_hypothesis`** | Log a standalone hypothesis node. | `workspace_id`, `text` |
| **`create_action`** | Log a standalone action node. | `workspace_id`, `text` |
| **`create_evidence`** | Log standalone evidence. | `workspace_id`, `text`, optional `source` |
| **`save_conclusion`** | Log conclusion linked to hypothesis + evidence IDs. | `workspace_id`, `text`, `outcome`, `tag`, optional `confidence`, `hypothesis_ids[]`, `evidence_ids[]` |

---

## Links

### Node links (hypothesis → action → evidence chain)

| Tool | What it does | Key params |
|------|----------------|------------|
| **`add_link`** | Link two nodes. Allowed pairs: hypothesis→action, action→evidence, conclusion→hypothesis, conclusion→evidence. | `workspace_id`, `from_type`, `from_id`, `to_type`, `to_id` |
| **`list_links`** | List all node links in workspace (get `id` for remove). | `workspace_id` |
| **`remove_link`** | Delete a node link. | `link_id` |

### Block links (block → block)

| Tool | What it does | Key params |
|------|----------------|------------|
| **`list_block_links`** | List block-to-block links. | `workspace_id` |
| **`remove_block_link`** | Delete a block link. | `link_id` |

Block links are also created via `save_block` → `link_to_block_ids`.

---

## Typical flows

### Log reasoning during a session
1. `list_workspaces` → pick workspace (or `create_workspace`)
2. Optional live capture: `start_capture` → `contextlayer-recorder watch` → `stop_capture` when done
3. `save_block` with title + hypothesis / action / evidence / conclusion as you go
4. `commit_checkpoint` at decision moments
5. `get_workspace_hygiene` before suggesting next steps

### Prep PR description
1. `list_blocks` → identify blocks for this PR
2. `export_blocks` with `block_titles: ["Fix auth refresh"]` or `block_ids: ["..."]`
3. Paste returned markdown into PR

### Edit / clean up
1. `get_block` by title to inspect
2. `save_block` with `block_title` + only changed fields
3. `delete_block` or `delete_workspace` when user asks to remove data

---

## Setup reminder

Cursor MCP config should point at the release binary:

```json
{
  "mcpServers": {
    "contextlayer": {
      "command": "C:\\Users\\miles\\ContextLayer\\target\\release\\contextlayer-mcp.exe"
    }
  }
}
```

After code changes: disable MCP → `cargo build -p contextlayer-mcp --release` → re-enable.

Optional env override: `CONTEXTLAYER_DB` (defaults to `~/.contextlayer/graph.db`).

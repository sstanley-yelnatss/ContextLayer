# Demo — paste into GitHub PR description

Copy this block after exporting from ContextLayer desktop or MCP `export_blocks`.

---

## PR Reasoning: fix MCP export ordering

**Workspace goal:** Ship true MVP with capture + trace CI.

### Block: Export blocks oldest-first

- **Hypothesis:** Reviewers read timeline top-to-bottom; oldest-first matches narrative.
- **Action:** Sort selected blocks by `created_at` in `compile_pr_reasoning`.
- **Evidence:** Manual export matches desktop timeline order.
- **Conclusion:** confirmed — merged sort into export crate.

### Hygiene

No open loops in exported selection.

---

*Exported from ContextLayer — 2 of 2 blocks*

# Scaffold: GitGraph-style fork/merge arcs on Session graph

**Status:** implemented (v1) — same-row fork/merge arcs in `SessionGraphView.tsx`.  
**Goal:** Show cross-lane fork/merge curves on the existing Session SVG (same idea as Cursor Git Graph), without touching Timeline, hygiene, MCP, or capture merge logic.

---

## Verdict

**Do it.** Session view already has lanes, `branch_fork` / `branch_merge` rows, and `merge_outcome`. The gap is rendering: today edges are **vertical within a lane only**. Cursor’s graph works because you *see* the branch leave and rejoin main.

Keep the current layout, dots, list, detail panel, and lane styling. Add arcs as a pure draw layer.

---

## What stays untouched

| Area | Why |
|------|-----|
| `build_session_graph` / `session_graph.rs` | Already emits fork/merge + outcomes |
| MCP (`merge_capture_branch`, etc.) | No API change |
| Timeline / block graph / hygiene | Different product surface |
| Row sort, `ROW_H` / `LANE_W`, selection, detail panel | Behavior unchanged |
| Capture storage / branch meta | Read-only for UI |

---

## Scope (v1)

**File:** `apps/desktop/src/components/SessionGraphView.tsx` only (ideally).

1. **Fork arc** — for each `branch_fork` row: curve from **main** lane `x` → branch lane `x` at that row’s `y` (same row index as today).
2. **Merge arc** — for each `branch_merge` row: curve from branch lane `x` → **main** lane `x` at that row’s `y`.
3. **Stroke** — use branch lane color (`laneStroke`).  
   - `merged_rejected` / `merge_outcome === "rejected"`: dashed + lower opacity.  
   - confirmed / active fork: solid, match existing stroke opacity language.
4. **Z-order** — draw arcs **under** dots (after verticals, before circles) so nodes stay readable.
5. **No click target on arcs** — selection stays on dots / list rows.

### Geometry sketch

```
x_main = laneIndex("main") * LANE_W + LANE_W/2
x_br   = laneIndex(row.lane) * LANE_W + LANE_W/2
y      = rowIndex * ROW_H + ROW_H/2 + GRAPH_PAD_TOP

// quadratic (GitGraph-ish elbow)
M x_main y  Q (x_main+x_br)/2 y  x_br y   // fork (same y → flat curve;
// if flat looks weak, use control point offset on y by ±ROW_H/3 for a visible hump)
```

Prefer a small vertical offset on the control point so the arc is visible (flat Q with same y is invisible). Match Cursor’s soft semicircle feel, not a hard right angle.

---

## Non-goals (v1)

- No React Flow / second graph engine  
- No arcs between arbitrary checkpoints  
- No changing message-range or checkpoint verticals  
- No new backend fields  
- No Timeline “reasoning graph” canvas  
- No hover tooltips on arcs (list + detail already explain)

---

## Risks & mitigations

| Risk | Mitigation |
|------|------------|
| Flat same-y curve invisible | Offset control point; visual check with 1–2 forks |
| Many parallel branches clutter | Cap visual weight (strokeWidth 1.5–2); reuse lane colors |
| Wrong main lane if missing | Skip arc if `laneIndex.get("main")` undefined |
| Rejected vs confirmed hard to tell | Dashed + opacity for rejected only |
| Accidental layout shift | Do not change row order, heights, or list markup |
| SVG width too narrow for arcs | Arcs stay inside existing lane columns; no width bump needed |

---

## Test plan (manual, after implement)

1. Workspace with **no** branches — graph identical to today.  
2. Active fork only — fork curve main → branch; branch verticals unchanged.  
3. `merged_confirmed` — merge curve into main; lane still dashed/faded as now.  
4. `merged_rejected` — merge arc dashed/dimmer; detail still shows rejected.  
5. Multiple branches — each fork/merge has its own arc; no crossed wrong lanes.  
6. Select fork/merge row — detail panel unchanged; active-head ring still works.  
7. Empty graph / start-capture CTA — unchanged.

Optional later: snapshot / small unit test for “path points for fork/merge rows” if we extract a pure helper.

---

## Implementation checklist (when approved)

- [x] Add `useMemo` building `{ kind, path, color, dashed }[]` from `graph.rows` + `laneIndex`  
- [x] Render `<path d=…>` under dots in existing `<svg>`  
- [x] Style rejected merges distinctly  
- [ ] Manual test plan above on `test capture` (or similar) with one confirmed merge  
- [x] No edits to Rust/MCP/types unless a bug is found (should not need)

---

## Open questions for review

1. **Fork source y:** always same row as `branch_fork`, or attach to nearest main-lane event at/near `main_log_seq_at_fork`?  
   - Recommend **same row** first (simplest, already aligned with list).  
2. **Should Timeline ever show this?** No for this change.  
3. **Animate merges?** No for v1 — static paths only.

---

## Approval gate

Implement only after explicit OK on:

- Frontend-only in `SessionGraphView.tsx`  
- Same-row main ↔ branch arcs  
- Rejected = dashed  
- Checklist + manual tests above

# ContextLayer — evaluation paper (working draft)

**Status:** Living research doc — add notes, data, and drafts here.  
**Goal:** GCC-style “with vs without tool” study showcasing ContextLayer strength.  
**Audience (v0):** Design partners, investors, blog — not peer review required initially.

**Related:** [FUTURE-IMPLEMENTATION.md](./FUTURE-IMPLEMENTATION.md) §9–§10 · [B1-PR-EXPORT-SPEC.md](./B1-PR-EXPORT-SPEC.md)

---

## Working title (TBD)

*Structured reasoning artifacts improve PR review comprehension and reduce undocumented dead ends in AI-assisted development*

Alt: *ContextLayer: reasoning receipts for human-in-the-loop review*

---

## One-sentence claim (draft)

Teams using ContextLayer (MCP logging + structured blocks + PR export) produce changes that reviewers understand faster and with fewer missed assumptions than diff-only review — without claiming raw SWE-bench pass-rate gains from the tool alone.

---

## Research questions

### Arm A — Reviewer wedge (primary)

1. Does attaching a ContextLayer PR export improve reviewer comprehension vs diff-only?
2. Does it reduce time-to-“I understand why this change was made”?
3. Do reviewers identify **rejected alternatives** more often when export includes hypothesis/action/evidence blocks?

### Arm B — Agent / SWE-adjacent wedge (secondary)

1. Same debugging or fix task: with vs without ContextLayer MCP — does task success differ?
2. With tool: are rejected paths documented in export at PR time (hygiene / completeness rubric)?
3. Token cost: tiered workspace index (G1) vs full workspace read — savings at equal task success?

---

## Hypotheses

| ID | Hypothesis |
|----|------------|
| H1 | Reviewers score higher on a structured comprehension quiz when PR includes reasoning export. |
| H2 | Median time-to-understand drops with export (same PR, same reviewer pool). |
| H3 | Agents with MCP logging produce exports that pass a “reasoning completeness” rubric more often than chat-only baseline. |
| H4 | Tiered MCP index (G1) reduces tokens vs naive full-read without hurting H3. |

---

## Experimental design (draft)

### Conditions

| Condition | Author workflow | Reviewer sees |
|-----------|-----------------|-------------|
| **Baseline** | Normal AI-assisted coding; PR description = usual (no ContextLayer) | Diff + standard PR body |
| **Treatment** | Same task; MCP logs blocks; author exports selected blocks to PR | Diff + ContextLayer markdown export |

### Tasks (pick 10–20)

- [ ] Dogfood PRs from ContextLayer repo (real)
- [ ] Synthetic bugfix scenarios (controlled diff + hidden rejected path in “author knowledge”)
- [ ] Optional: 2 external design-partner PRs (anonymized)

### Reviewer protocol

- [ ] Recruit N reviewers (mix: semi-technical + mid SWE — **not** only senior SWE)
- [ ] Within-subjects or between-subjects — **TBD**
- [ ] Comprehension quiz per PR (e.g. “What was tried first?” “Why was path X rejected?” “What evidence supports the fix?”)
- [ ] Record time to submit quiz
- [ ] Optional: LLM-as-judge sanity check on same rubric

### Agent protocol (Arm B)

- [ ] Harness: same repo/issue, two runs (MCP off vs on)
- [ ] Fixed model + IDE where possible
- [ ] Outcomes: tests pass (Y/N), export attached (Y/N), rubric score, open hygiene count at export time

---

## Metrics

| Metric | Arm | How measured |
|--------|-----|--------------|
| Comprehension score | A | Quiz / rubric 0–100 |
| Time to understand | A | Seconds to quiz submit |
| Rejected path identified | A | Y/N on quiz item |
| Task success | B | Tests / issue closed |
| Reasoning completeness | B | Rubric on export (see below) |
| Token usage | B | MCP logs: index vs full read |

### Reasoning completeness rubric (draft)

Score 0–5 each; sum /25:

- [ ] Hypothesis or goal stated
- [ ] At least one rejected alternative documented
- [ ] Evidence linked to conclusion
- [ ] Belief/conclusion explicit
- [ ] No critical open loops in **exported** selection (or flagged in footer)

---

## Baselines & related work (notes)

**GCC:** SWE-bench with vs without graph context compiler — cite when published/OSS reviewed.

**Others (positioning, not necessarily same benchmark):**

- Raw chat logs / session export tools
- Copilot Code Review (bug finding, not reasoning path)
- Langfuse / LangSmith (app traces, not PR reasoning receipt)

**Our differentiation in paper:** We measure **human review comprehension** and **structured reasoning completeness**, not only automated task resolution.

---

## Limitations (pre-write)

- Small N for v0
- Author may curate export (selection bias) — discuss honestly; footer hygiene helps
- LLM-as-judge bias if used
- Senior SWE reviewers may show smaller effect (see ICP notes in FUTURE-IMPLEMENTATION §11)

---

## Results (paste data here)

### Run log

| Date | Task ID | Condition | Reviewer | Comprehension | Time (s) | Notes |
|------|---------|-----------|----------|---------------|----------|-------|
| | | | | | | |

### Arm B run log

| Date | Task ID | MCP on? | Tests pass | Rubric /25 | Tokens | Notes |
|------|---------|---------|------------|------------|--------|-------|
| | | | | | | |

---

## Figures & tables (TODO)

- [ ] Bar chart: comprehension baseline vs treatment
- [ ] Box plot: time to understand
- [ ] Table: ablation G1 tiered vs full read tokens

---

## Draft outline (paper / long blog)

1. Abstract
2. Introduction — AI PR volume, reasoning gap
3. Related work — GCC, Agent Trace, review tools
4. ContextLayer treatment — blocks, hygiene, export, MCP
5. Methods — Arm A + Arm B
6. Results
7. Discussion — ICP, enterprise implications, limitations
8. Conclusion

---

## Scratch notes (add freely)

<!-- Miles: paste ideas, quotes, mentor feedback, GCC notes below -->



---

## External reference to emulate (OneContext)

**Status:** TBD — paste paper URL / PDF notes here when located.

When adding:
- What they measured (comprehension? task success? tokens?)
- What we copy vs what ContextLayer does differently (reasoning graph + PR export, not chat versioning)
- Do **not** create a second benchmark doc in Samble repo — all study design stays in this file.

---

## Changelog

| Date | Change |
|------|--------|
| 2026-06-07 | Initial scaffold from competitive / mentor discussion |

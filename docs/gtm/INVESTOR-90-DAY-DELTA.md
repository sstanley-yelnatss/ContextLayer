# ContextLayer — 90-day investor delta

**Purpose:** Concrete build + traction plan to move from “clever devtool” to fundable **AI change governance** story.  
**Audience:** You, design partners, angels, YC-style devtools / AI-security programs.  
**Baseline (2026-06):** v1.2+ shipped — desktop app, MCP, PR markdown export, hygiene panel, local SQLite **plus capture/trace lane** (see below).  
**Investor one-liner:** *Reasoning receipt on every AI-assisted PR — GitHub-native governance, not another notes app.*

**Companion:** [PITCH.md](./PITCH.md) · [POSITIONING.md](./POSITIONING.md) · [PRD-addendum-merged-vision.md](./PRD-addendum-merged-vision.md)

---

## Already shipped — do not re-plan as 90-day build

The capture/governance lane is **built locally** (dogfood + rebuild pending per [.cursor/IMPLEMENTATION-PLAN.md](../.cursor/IMPLEMENTATION-PLAN.md)):

| Component | Status |
|-----------|--------|
| **`contextlayer-recorder`** — opt-in live capture from Cursor transcripts | ✅ Shipped |
| **Checkpoints + branches + merge** — decision moments, fork/merge capture lines | ✅ Shipped |
| **PR trace appendix** — checkpoints on by default; optional raw log (**first 50 msgs since capture start**) | ✅ Shipped |
| **`crates/trace`** — JSONL store, `redact_secrets`, context reads | ✅ Shipped |
| **Trace CI rules** — `.contextlayer/rules.yml` + `run_trace_check` in `crates/trace` | ✅ Shipped |
| **`contextlayer-trace` CLI** — local trace check runner | ✅ Shipped |
| **MCP** — `start_capture` / `stop_capture`, `commit_checkpoint`, branch/merge, trace in `export_blocks` | ✅ Shipped |
| **Desktop** — session trace UI (checkpoints + raw log), live capture controls | ✅ Shipped |

**90-day focus for this lane:** wire to **GitHub** (Action/App), dogfood on real PRs, design-partner metrics — **not** rebuilding the recorder.

**Still deferred (OK to slip past 90 days):** `cleanup_capture`, GitHub App/bot, hosted trace store, enterprise SSO. Co-founder lane = **GitHub integration + hosted ops**, not greenfield capture.

---

## What investors need to see (vs today)

| Gap today | What closes it |
|-----------|----------------|
| No revenue / pricing | Published team tier + 3 design partners on “pilot agreement” |
| Capture exists locally, not in CI | **GitHub Action** wrapping existing `trace-cli` / rules check |
| Dual pitch (investigator + DevOps) | **Agent DevOps only** in deck; investigator = dogfood |
| Policy-as-code exists but not on PRs | Action reads `.contextlayer/rules.yml` on every PR |
| No before/after data | Review time + artifact attach rate from 2–3 teams |

**Comparable framing:** Snyk / Semgrep (merge gate) + judgment receipt (GuardSpine / Verity) — not Obsidian, not chat logging.

---

## 90-day outcomes (exit criteria)

By **Day 90**, you should be able to say:

1. **≥3 eng teams** (or ≥15 devs across teams) using PR export on **real PRs** at least weekly.
2. **≥50% of AI-tagged PRs** in pilot repos have a reasoning artifact attached (description or committed file).
3. **GitHub Action** live: checks for artifact presence; optional fail on high-risk paths.
4. **Before/after** on ≥1 team: median review time or reviewer confidence (even n=20 PRs is fine early).
5. **Pricing page** live: free solo / paid team / enterprise contact.
6. **One compliance mapping doc** (EU AI Act traceability, SOC 2 change management) — no cert required.

If you hit 4/6, you’re credible for angels and accelerator apps with a devtools angle. Hit 6/6 for seed conversations.

---

## Build plan (by month)

### Days 1–30 — Workflow wedge

| # | Ship | Effort | Why |
|---|------|--------|-----|
| 1 | **GitHub Action v0** — call existing trace check (`contextlayer-trace` / `run_trace_check`) + PR body / `docs/reasoning/*.md` markers | M | Ships local governance to where merges happen |
| 2 | **PR template snippet** + `gh` script to paste export (blocks + optional trace appendix) into description | S | Zero App review; demo tomorrow |
| 3 | **Export bundle v1** — markdown + `manifest.json` (block IDs, timestamps, SHA-256 chain) | M | “Judgment receipt,” not chat log |
| 4 | **Path-based rules in Action** — extend shipped `TraceRules` (e.g. `auth/` → `require_checkpoint`) | S | Policy-as-code already in crate; wire + document |
| 5 | **Landing + pricing** — one page, Agent DevOps only, “Request design partner” | S | Fundraising requires a URL |
| 6 | **Recruit 5 design partner leads** — Cursor-heavy teams, 5–50 eng, outbound + network | — | Traction > features |

**Day 30 metrics**

| Metric | Target |
|--------|--------|
| Design partners signed (LOI or active) | 3 |
| Real PRs with export attached | 10+ |
| GitHub Action installs | 2 repos |

---

### Days 31–60 — Enforcement + measurement

| # | Ship | Effort | Why |
|---|------|--------|-----|
| 7 | **GitHub App (minimal)** — comment on PR open with link if export detected; pin template | L | Discoverability; reviewers see receipt in Conversation |
| 8 | **Risk tiers in rules** — `auth/`, `payments/`, `**/migrations/**` → `gate: require_export` | M | AUTO / GATE narrative (Verity-style) |
| 9 | **Team dashboard v0** — spreadsheet or simple static site: repos, % PRs with artifact, last export date | M | Investor slide: adoption curve |
| 10 | **Reviewer feedback** — GitHub comment template: 👍 / 👎 “reasoning helped” on pilot PRs | S | Qualitative + quant signal |
| 11 | **Compliance mapping v1** — 1-pager: which controls artifact satisfies (change mgmt, traceability) | S | Enterprise procurement language |
| 12 | **Case study #1** — one team, one PR, before/after narrative + screenshot | S | Deck content |

**Day 60 metrics**

| Metric | Target |
|--------|--------|
| Weekly active exporters (devs) | 8+ |
| % pilot PRs with artifact | 40%+ |
| Median review time delta | Documented (even directional) |
| Paying or committed pilot ($) | 1 team @ $X/mo OR 3 teams on free with renewal intent |

---

### Days 61–90 — Fundraise-ready package

| # | Ship | Effort | Why |
|---|------|--------|-----|
| 13 | **Stripe / manual billing** for Team tier ($15–25/dev/mo ballpark) | M | Revenue line on deck |
| 14 | **Org policy sync** — rules.yml in repo root; Action reads it | M | “Deploy like Snyk” |
| 15 | **End-to-end dogfood demo** — capture → checkpoint → `export_blocks` (trace appendix) → PR → Action pass | M | Proves dual-lane story on one real PR |
| 16 | **10-slide deck** — problem, wedge, demo GIF (include trace panel), metrics, comps, ask | S | Application asset |
| 17 | **Second case study** — different team or repo | S | Pattern, not one-off |

**Day 90 metrics**

| Metric | Target |
|--------|--------|
| MRR or committed ARR | $500+ (symbolic OK if design partners convert) |
| Teams on weekly habit | 3 |
| GitHub stars / waitlist | 200+ stars OR 100+ waitlist (optional social proof) |

---

## What NOT to build in 90 days

- **Rebuilding capture/trace** — recorder, checkpoints, 50-msg appendix, redaction, and local trace CI already shipped
- Cross-AI sync (Phase 4)
- `cleanup_capture`, git worktree bridge, hosted trace store (see IMPLEMENTATION-PLAN §E)
- Graph visualization, mobile, cloud sync
- Investigator-as-primary GTM
- Enterprise SSO (mention on enterprise tier; build after 10 teams)

---

## Pricing sketch (publish by Day 30)

| Tier | Price | Includes |
|------|-------|----------|
| **Solo** | Free | Desktop + MCP + local capture/recorder |
| **Team** | $15–25/dev/mo | GitHub Action, org rules, usage dashboard |
| **Enterprise** | Contact | SSO, audit export, custom rules, hosted trace retention |

Design partners: **free 90 days** in exchange for weekly metric share + logo OK.

---

## Investor deck — slides to add

1. **AI PR volume ↑, review trust ↓** — 1 stat (GitHub Copilot % or internal pilot)
2. **Diff vs reasoning receipt** — side-by-side screenshot (blocks + optional trace appendix)
3. **GitHub Action in 60 seconds** — install GIF
4. **Policy-as-code** — sample `rules.yml` (already implemented locally)
5. **Traction** — % PRs with artifact, review time chart
6. **Comps** — Graphify / Copilot Review / Snyk / Verity (we own *why*)
7. **Business model** — seat-based, land in eng, expand to security/compliance
8. **Ask** — $X pre-seed OR design partners + accelerator

---

## Weekly operator rhythm

| Day | Action |
|-----|--------|
| Mon | Check pilot repos: % PRs with export |
| Wed | 1 design partner call (15 min): what blocked attach? |
| Fri | Ship one small thing; post changelog / Twitter / HN “Show HN” prep |

---

## Accelerator fit (apply when)

| Program type | Apply when |
|--------------|------------|
| YC / devtools | Day 60+ with 3 teams + Action demo |
| AI security / governance | Day 30+ if compliance mapping + Action live |
| EU programs | Weaker fit unless EU AI Act angle leads |

**Do not apply** with only desktop app + pitch — you need GitHub + 2 teams minimum.

---

*ContextLayer investor delta v1.1 — 2026-06 (baseline corrected: capture/trace shipped)*

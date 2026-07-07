# ContextLayer — distribution steals checklist

**One-pager.** Patterns from adjacent startups, mapped to what we can run now.  
**Status:** Friends beta → design partners → public narrative.  
**Last updated:** 2026-06-07

---

## North star (don’t forget)

- **Buyer:** EM / platform / security (review throughput + audit), not solo senior SWE.
- **User:** Dev in Cursor logging via MCP + desktop.
- **Wedge:** PR reasoning export + optional capture/trace — *why*, not just diff.
- **Success signal:** Someone exports to a **real PR** twice without you nagging; a reviewer says it helped.

---

## Phase 0 — Ready to invite (this week)

| # | Steal from | Action | Done |
|---|------------|--------|------|
| 0.1 | **Graphite** (one team champion) | Co-founder demo + **1** external design partner on a 30-min setup call | ☐ |
| 0.2 | **Stripe** (docs = distribution) | `docs/MCP-SETUP.md` + 5-min “first PR export” path is the onboarding | ☐ |
| 0.3 | **Notion** (templates) | Demo workspace + `examples/demo-pr-reasoning.md` in repo; clone → export in minutes | ☐ |
| 0.4 | **PostHog** (open core) | README “friends beta” expectations; optional **waitlist** line for team tier (email capture only) | ☐ |
| 0.5 | **Windows .exe** | `npm run tauri build` → send unsigned installer to non-dev testers | ☐ |

---

## Phase 1 — Credibility (2–4 weeks)

| # | Steal from | Action | Done |
|---|------------|--------|------|
| 1.1 | **GCC** (benchmark paper) | Mini reviewer study: 5–8 PRs, 8–12 reviewers, diff-only vs diff + export → blog post from [EVAL-PAPER.md](./EVAL-PAPER.md) | ☐ |
| 1.2 | **Snyk / Codecov** (CI = billboard) | Dogfood trace CI on `develop` → `main` PR; paste export in description | ☐ |
| 1.3 | **Case study** (every B2B devtool) | 1-page PDF: one team, before/after PR description (anon OK) | ☐ |
| 1.4 | **Codecov** (badge) | Optional README badge: “Trace CI” / link to Action template when public | ☐ |

---

## Phase 2 — Public narrative (ongoing)

| # | Steal from | Action | Cadence |
|---|------------|--------|---------|
| 2.1 | **Cursor** (build in public) | LinkedIn: problem posts for EM/founders; X: 60s export screen recording | 2–3× / week |
| 2.2 | **Category creation** | Publish [COMPARISON-vs-chat-log-tools.md](./COMPARISON-vs-chat-log-tools.md) (LinkedIn long-form or blog) | Once, then slice into threads |
| 2.3 | **Complement positioning** | Second post: “ContextLayer + Graphify” (stack layer, not fight) | After comparison lands |
| 2.4 | **Vercel / “Sent from iPhone”** | PR export footer links to one-pager (“what is this markdown?”) | When site/Notion page exists |
| 2.5 | **MCP ecosystem** | List in awesome-MCP / Cursor community when demo video exists | One-time + maintain |

---

## Phase 3 — Scale signals (when Phase 1 hits)

| # | Steal from | Action | Trigger |
|---|------------|--------|---------|
| 3.1 | **Snyk** (dev → security budget) | Pitch trace CI + rules.yml to platform/security leads | 2 teams using export on real PRs |
| 3.2 | **Graphite** (team mandate) | PR template: “Attach ContextLayer export” for one squad | Champion agrees |
| 3.3 | **PostHog** (paid waitlist) | “Team tier: sync + GitHub bot + SSO” — LOI / “would you pay $X?” on calls | 3+ partners |
| 3.4 | **HN / Product Hunt** | Launch spike | After .exe + eval post + 1 case study |
| 3.5 | **Newsletter guest post** | Eng management / AI eng newsletter pitch | After eval numbers |

---

## Willingness-to-pay (no Stripe yet)

Ask on every design partner call:

1. “If this saved **2 hours per review week** on your team, would **$20/seat/mo** be obvious, maybe, or no?”
2. “What would **team tier** need: shared workspace, GitHub bot, required CI, read-only reviewer link?”
3. **Kill signal:** They stop exporting after week 2 and go back to paste-in-chat only.

---

## Do not do yet

- Paid ads, full hosted SaaS, Obsidian vault export, Product Hunt before polish
- Selling primary ICP = senior SWE who builds their own publisher agent
- Block fork UX (dropped — use block links + capture branch)

---

## Quick links

| Doc | Use |
|-----|-----|
| [PITCH.md](./PITCH.md) | Read-off / co-founder |
| [POSITIONING.md](./POSITIONING.md) | Hero lines + vs chat-log |
| [EVAL-PAPER.md](./EVAL-PAPER.md) | Benchmark protocol |
| [BETA-LAUNCH-CHECKLIST.md](./BETA-LAUNCH-CHECKLIST.md) | Repo public / friends beta |
| [COMPARISON-vs-chat-log-tools.md](./COMPARISON-vs-chat-log-tools.md) | First comparison post draft |

---

## Changelog

| Date | Change |
|------|--------|
| 2026-06-07 | Initial one-pager from startup playbook review |

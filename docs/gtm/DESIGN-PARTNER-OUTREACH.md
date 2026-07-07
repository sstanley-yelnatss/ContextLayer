# ContextLayer — design partner outreach (operator plan)

**Status:** Active — first 30 days of distribution  
**Last updated:** 2026-06-29  
**Related:** [../product/PITCH.md](../product/PITCH.md) · [../product/POSITIONING.md](../product/POSITIONING.md) · [../implementation/BETA-LAUNCH-CHECKLIST.md](../implementation/BETA-LAUNCH-CHECKLIST.md) · [EVAL-PAPER.md](./EVAL-PAPER.md) · [../product/FUTURE-IMPLEMENTATION.md](../product/FUTURE-IMPLEMENTATION.md) §11

---

## 0. What exists elsewhere (and what this doc is)

| Doc | Has | Missing |
|-----|-----|---------|
| [BETA-LAUNCH-CHECKLIST.md](../implementation/BETA-LAUNCH-CHECKLIST.md) | Friends beta setup, 10-min first session | No channel plan, no cadence |
| [../product/FUTURE-IMPLEMENTATION.md](../product/FUTURE-IMPLEMENTATION.md) §11 | ICP, kill signals, “5 design partners” | No weekly ops, no copy |
| [EVAL-PAPER.md](./EVAL-PAPER.md) | Study design | Recruitment as magnet, not ops |
| [../product/PITCH.md](../product/PITCH.md) | Read-off narrative | Not executable |

**This doc = the 30-day operator playbook.** One metric, one ask, concrete channels, no guru fluff.

---

## 1. North star (only metric that matters right now)

> **Design partners who attach a ContextLayer export to a real merged PR without you reminding them.**

Not followers. Not stars. Not “interested DMs.”

| Signal | Meaning |
|--------|---------|
| **Win** | Partner exports on PR #2 unprompted; reviewer says it helped |
| **Maybe** | Partner exports on PR #1 after your white-glove session |
| **Kill** | Partner reverts to chat paste in PR after week 2 |
| **Ignore** | “Cool tool!” with no setup call booked |

**Target for day 30:** 3 onboarded partners, **2** with export on ≥1 merged PR, **1** reviewer quote you can cite.

---

## 2. The ask (keep it stupidly small)

You are **not** selling ContextLayer. You are recruiting **4-week design partners**:

1. 30-min setup call (you do the install)
2. Use on **one real PR** in week 1 (you sit on the export if needed)
3. Second PR in week 2–3 **alone**
4. 15-min debrief or async voice note after each PR

**Offer:** Free, early access, direct line to you, shape the roadmap. No equity, no logo requirement, no NDA.

**Do not ask for:** “Try it when you can,” “share with your network,” “feedback on the vision,” multiple products, or “be an advisor.”

---

## 3. Who to pursue (and who to ignore)

### Pursue (in order)

| Tier | Who | Where they hang out |
|------|-----|---------------------|
| **A** | IC or EM at **5–30 person** startup, Cursor daily, ≥2 AI-heavy PRs/week | Your network, YC alumni Twitter, indie founder Discords |
| **A** | **Semi-technical founder** who ships their own code | Twitter, Indie Hackers, founder group chats |
| **B** | **Platform / dev-infra** person rolling out Cursor team-wide | LinkedIn, eng managers |
| **B** | Security / bounty **investigator** (branch/merge pitch) | Bounty Discords, security Twitter |

### Ignore (politely)

- Senior SWE who already built a personal “publisher agent”
- People with zero PR reviewers (solo side project with no merge discipline)
- Anyone who won’t book a setup call
- Clinic / non-dev buyers (PrivacyOS lane — wrong product)

### Qualification (10 min, before onboarding)

1. How many PRs last week with heavy AI use?
2. Who reviews — teammate or async?
3. What goes in PR description today?
4. Will you do 30 min setup + 2 PRs in 4 weeks?

Need **≥2 PRs/week** and a **human reviewer** for the Agent DevOps wedge.

---

## 4. Thirty-day operator calendar

### Week 0 — Showroom (before loud outreach)

Do this once. Everything else depends on it.

- [ ] **Dogfood PR on ContextLayer repo** with reasoning export in description (link in PR)
- [ ] **60-sec Loom** (no music, no hype): capture start → log block → branch → export → paste in PR
- [ ] **3 screenshots** in README (timeline, hygiene, export)
- [ ] **Invite message** copied from §8 below into Notes app
- [ ] Spreadsheet: `partner | source | call date | PR1 | PR2 | reviewer quote`

Do **not** post Show HN or big Twitter thread until Loom + dogfood PR exist.

### Week 1 — Warm only (goal: 2 onboarded)

| Day | Action |
|-----|--------|
| Mon | List 15 warm names (ex-coworkers, founder friends, “ships Cursor PRs”) |
| Tue–Thu | 5 DMs/day — template §8, book setup calls |
| Fri | Run 2 setup calls; partners create workspace on **their** active task |

**Rule:** No cold LinkedIn until 1 warm partner has exported on a real PR.

### Week 2 — Twitter + eval magnet (goal: 1 more onboarded)

| Day | Action |
|-----|--------|
| Mon | Tweet #1 — problem (context rot) — §7 |
| Wed | Tweet #2 — 60-sec demo clip |
| Fri | Tweet #3 — design partner ask — §7 |
| Ongoing | Reply to 3 posts/day from Cursor/AI eng accounts (helpful, not pitch) |
| Ongoing | 2 cold DMs/day to EMs at 10–50 person startups |

Optional: post eval study recruit — [EVAL-PAPER.md](./EVAL-PAPER.md) — “author + reviewer pairs, 2 PRs, I share results.”

### Week 3 — Double down on what worked

- If warm DMs convert → do more warm, not more channels
- If Twitter DM inbound → pin design partner tweet
- If security person bites → post one investigator-thread (branch/merge rabbit hole)
- Run partner check-in: “Did export land? What did reviewer say?”

### Week 4 — Decide

| Result | Next move |
|--------|-----------|
| 2+ unprompted exports | Prep Show HN; ask partners for public quote; widen cold |
| 1 export, you nagged | Fix onboarding friction; watch next 2 partners |
| 0 exports | Stop broadcasting; sit with 3 users on calls until you see why |

---

## 5. Channel playbook (ranked)

### 5.1 Warm DMs — highest ROI

5 messages/day max. Personalized first line. Same ask every time.

See §8 templates.

### 5.2 Twitter/X — distribution, not branding

**Profile (one line):** Building ContextLayer — reasoning receipts for AI-assisted PRs. Local-first. Looking for design partners.

**Content mix (4 posts/week):**

| Type | Ratio | Purpose |
|------|-------|---------|
| **Problem** | 1/week | Context rot, reviewer sees diff not path |
| **Build in public** | 1/week | Shipped X, learned Y (specific) |
| **Demo** | 1/week | Loom or screen recording, no voiceover required |
| **Ask** | 1/week | Design partners — explicit CTA |

**Reply game (daily, 15 min):** Find posts about AI PR spam, bad reviews, “my team uses Cursor.” Add one useful sentence. Pitch only if they reply or DM.

**Do not:** Thread storms, “excited to announce,” engagement bait, compare yourself to Microsoft, post vision without demo.

### 5.3 Show HN — week 3–4 only

**When:** Dogfood PR + Loom + README screenshots live.

**Title:** `Show HN: ContextLayer – structured reasoning exports for AI-assisted PRs`

**First comment (post immediately):** What it is, what’s local, what you want (design partners), honest limitations (Windows-first, build from source).

HN is a spike, not a strategy. One day of traffic; capture emails/DMs manually.

### 5.4 LinkedIn — cold EM outreach

3–5 messages/week. Short. Link Loom, not repo.

Angle: **reviewer time**, not “governance platform.”

### 5.5 Cursor / dev communities

One genuine post in Cursor forum or Discord after you have Loom. “Built X, looking for 2 teams to try PR export — not selling.”

### 5.6 Eval paper as recruitment

Tweet or DM:

> Running a small study: PR + reasoning export vs diff-only — does review comprehension improve? Need 3 author/reviewer pairs. 30 min setup, 2 PRs, early access + I’ll share results.

Converts researchers and EMs who ignore product pitches.

---

## 6. Design partner ops (what you do on the call)

### Setup call (30 min)

1. Their **active PR or task** — create workspace with real goal on call
2. Desktop + MCP + `bind-repo` + `start capture` (you drive screen share)
3. Log one block via MCP live
4. Show export → “this goes in PR description”
5. Book async check-in for **day they open PR**

### After PR merges

- Slack/WhatsApp: “Did reviewer mention the export?”
- If no export attached: 10-min “do it together” call — no shame, just ops
- Log row in spreadsheet

### What you owe them

- Same-day response on setup bugs
- Priority fix if export blocks their workflow
- Credit in changelog if they want it

---

## 7. Tweet drafts (edit voice, keep structure)

**Problem (context rot):**

> AI-assisted coding means more PRs. Reviewers still only see the diff. The hypothesis, the rejected path, the evidence — stuck in a private Cursor chat. That's context rot. We're building the layer between that chat and the PR.

**Build in public:**

> Shipped capture branch/merge in ContextLayer — when the agent rabbit-holes, fork the session, merge when you pick a path. Next: every PR on our repo gets a reasoning export attached. Dogfooding the governance layer before we pitch it.

**Demo (attach Loom):**

> 60 sec: Cursor session → structured blocks → PR markdown export. Local SQLite, MCP logs while you work. Not a chat logger — reasoning graph (hypothesis / evidence / conclusion). [link]

**Design partner ask:**

> Looking for 3 design partners: teams shipping 2+ Cursor-heavy PRs/week. I'll white-glove setup (30 min). You attach our reasoning export to 2 real PRs over 4 weeks. Free, early access, direct line to me. DM if review bottleneck is real.

**Investigator angle (alternate week):**

> Long AI-assisted security investigations have the same failure mode as eng: you lose the thread, retest dead paths, can't explain why you're confident. We branch capture when the agent veers off. Looking for 2 bounty folks to dogfood.

---

## 8. DM templates

**Warm:**

> Hey [name] — quick ask. I'm building ContextLayer: local reasoning graph for Cursor work that exports a "why we did this" block for PRs (not chat dumps). Capture + git-style session branching is working. Looking for 2 design partners this month — 30 min setup with me, try attaching export to 2 real PRs, tell me if your reviewer cared. Free, no strings. Worth it for you?

**Cold (EM / lead):**

> Hi [name] — saw [company] is shipping fast. Quick question: are AI-assisted PRs creating a review bottleneck? I built a local tool that attaches a structured reasoning receipt to PRs (hypothesis → evidence → conclusion). Looking for 2 design partners — I do 30 min onboarding, you try it on 2 PRs over a month. No cost. 60-sec demo: [Loom]. If this isn't your problem, ignore — but if reviewers only see diffs, might be worth 15 min.

**After they say yes:**

> Great — book here [Cal link] or reply with 2–3 times. Before call: have one active task/PR in mind. I'll screen-share setup; you'll leave with a workspace tied to real work. 30 min.

---

## 9. Positioning on outbound (what to say vs shut up about)

### Say

- Reasoning receipt on the PR
- Reviewer understands *why*, not just *what*
- Local-first, structured graph, not chat history
- Capture + branch when agent goes wrong direction
- Design partner — you want honest friction

### Shut up about (until they ask)

- “Governance layer for autonomous agents”
- Microsoft / Nadella quotes
- Full platform vision / GitLLM merge
- “No one else has this” (let them conclude)
- Enterprise, SSO, compliance (later buyer)

Lead with **reviewer pain**. Expand to governance when they're hooked.

---

## 10. Weekly scorecard (Friday, 10 min)

| Metric | Target |
|--------|--------|
| Setup calls booked | ≥1/week early |
| Setup calls completed | ≥1/week |
| Partners with export on merged PR | cumulative toward 2 |
| Unprompted export (no nag) | cumulative toward 1 |
| Reviewer quotes captured | ≥1 by day 30 |
| DMs sent (warm + cold) | 15–25/week |
| Tweets posted | 3–4/week |
| Reply engagements (helpful) | 10–15/week |

If calls booked = 0 for 2 weeks, problem is **ask or demo**, not channel volume.

---

## 11. Anti-patterns (guru slop to avoid)

| Slop | Do instead |
|------|------------|
| “Building in public” with no demo | Ship dogfood PR first |
| “Revolutionizing AI development” | One sentence: export on PR |
| Posting daily vision threads | One demo video beats ten threads |
| “Join waitlist” with no onboarding | Book setup calls |
| Optimizing profile / landing page | 15 DMs beat new website |
| Targeting “AI engineers” broadly | 5–30 person startup, Cursor, has reviewers |
| Free for everyone forever | Free for **4 partners**, then narrow |

YC operator truth: **do things that don't scale.** You are the onboarding team until export-on-PR-2 happens without you.

---

## 12. Assets checklist

| Asset | Status | Blocks outreach? |
|-------|--------|------------------|
| Dogfood PR with export | [ ] | **Yes** |
| 60-sec Loom | [ ] | **Yes** |
| README screenshots | [ ] | Helps |
| Calendly / booking link | [ ] | Helps |
| Partner spreadsheet | [ ] | Ops |
| Windows `.exe` (optional) | [ ] | No — source OK for design partners |

---

## 13. After day 30

**If signal is good:**

- Show HN
- Ask best partner for case study tweet
- Expand to 5 partners max (don't dilute support)
- Start eval paper data collection for blog/investor

**If signal is weak:**

- Do not increase ad spend or content volume
- Run 5 user calls: “walk me through your last PR without ContextLayer”
- Fix top onboarding blocker (usually MCP setup or export friction)
- Consider investigator wedge if eng wedge stalls

---

## Changelog

| Date | Change |
|------|--------|
| 2026-06-29 | Initial operator plan — fills gap left by beta checklist + FUTURE-IMPLEMENTATION |

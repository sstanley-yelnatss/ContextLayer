# Others version chat. We version reasoning — and ship it to the PR.

**Draft for LinkedIn long-form, blog, or newsletter.**  
Edit voice as needed; keep the category line.

---

## Hook (use as opening)

Your team ships more pull requests than ever because AI helps write the code. Reviewers still open the PR and see a diff. The story of what was tried, what failed, and why this fix won lives in a private Cursor chat somewhere. That gap is where review slows down and trust breaks.

Chat-log tools promise to fix this by saving the conversation. ContextLayer takes a different bet: **reviewers don’t need the whole chat. They need the reasoning receipt.**

---

## What “chat-log tools” optimize for

Tools in this bucket (session importers, conversation branch UIs, compile-my-history-for-the-agent products) are built around one unit of value: **the transcript**.

| They optimize for | Typical output |
|-------------------|----------------|
| Storing what was said | Searchable chat, branches, trees |
| Feeding the agent again | Compiled context packet for the *next* prompt |
| Session continuity | “Pick up where you left off” for *you* |

That helps the person *in* the session. It does not automatically help the person *reviewing the PR*, who was never in the room.

---

## What ContextLayer optimizes for

ContextLayer is a **local-first reasoning graph** plus an optional **capture lane** for live AI work.

| Layer | What it is | Who it serves |
|-------|------------|---------------|
| **Blocks (H/A/E/C)** | Curated timeline: hypothesis, action, evidence, conclusion | You + reviewers |
| **Hygiene** | Orphans, stale threads, dead ends flagged | You (so nothing gets lost) |
| **PR export** | Selected blocks → markdown → paste in PR description | Reviewers on GitHub |
| **Capture + branch/merge** | Opt-in session log; fork conversation tangents; fold back to main | Audit + “what we explored” |
| **Checkpoints** | Decision commits on the capture line | Governance / merge gates |
| **MCP + skill** | Agent logs into the same graph while you code in Cursor | Live workflow |

The unit of value is not “more chat saved.” It is **structured belief over time**, exported where review actually happens.

---

## Side-by-side

| | Chat-log / session tools | ContextLayer |
|---|--------------------------|--------------|
| **Primary artifact** | Conversation tree or transcript | Reasoning blocks + PR markdown |
| **Review surface** | You bring the reviewer to the tool (or dump chat) | You bring the reasoning to **GitHub** |
| **Structure** | Messages in order | Hypothesis → evidence → conclusion (any subset) |
| **Open loops** | Easy to lose in long threads | Hygiene panel + belief states |
| **Agent context** | Often the main event | MCP logging + tiered reads; export is the handoff |
| **Storage** | Often cloud/session-centric | Local SQLite; markdown is a view |
| **Tangents** | Branch the chat | Capture branch/merge *and* linked blocks |
| **Compliance story** | “We kept the logs” | “Here’s the reasoning receipt + optional trace CI” |

---

## When chat-log tools are the right tool

Use them (or plain chat search) when:

- You are the only reader and you want full verbatim history.
- Your goal is to **resume** a long session with the same agent.
- You are researching compile strategies for **token-efficient agent context** (GCC-style), not human PR review.

---

## When ContextLayer is the right tool

Use it when:

- A **human reviewer** needs to understand *why* a change is believable without reading 200 messages.
- You run **AI-assisted PRs at volume** and the bottleneck is trust, not typing speed.
- You want **open loops visible** (rejected paths, unsettled hypotheses) before merge.
- You need a path toward **policy** (“reasoning artifact on PRs”) without shipping chat logs to the cloud.

---

## How they fit together (not either/or)

ContextLayer is not anti-transcript. Capture can ingest live Cursor chat into a local log. Branch/merge handles conversation tangents. **Export for PR** still leads with **curated blocks**, because that is what reviewers read in 5 minutes on GitHub.

Think of it as two layers:

```
AI session  →  capture (optional, local)  →  blocks (curated)  →  PR export
                    ↑                              ↑
              chat-log tools                 ContextLayer wedge
              often stop here                ships here
```

Chat-log tools help you **remember the conversation**. ContextLayer helps your **team approve the change**.

---

## One line for the footer

> **Others version chat or agent logs. We version reasoning — and ship it to the PR.**

Category: **AI change governance** · PR reasoning · human-in-the-loop review for agent-assisted development.

---

## CTA (pick one)

- **Friends beta:** Repo public — clone, MCP setup, 30-min first export. [Link README]
- **Design partners:** DM if you review AI-heavy PRs and want a setup call.
- **Waitlist:** Team tier (sync, GitHub bot, trace CI templates) — [email/form]

---

## Social slices (after publishing long-form)

**Twitter/X thread (5 tweets):**

1. Reviewers see the diff. The “why” is still in someone’s Cursor chat. That’s the AI PR bottleneck.
2. Chat-log tools save the transcript. Useful for you. Doesn’t automatically help the reviewer on GitHub.
3. ContextLayer: hypothesis / action / evidence / conclusion blocks → export selected blocks into the PR description.
4. Optional capture + branch/merge for tangents. Hygiene flags dead ends so they don’t vanish.
5. Local-first, MCP in Cursor. Others version chat. We version reasoning — and ship it to the PR. [link]

**LinkedIn (short):**

We ship 3× more AI-assisted PRs. Reviewers still only see diffs. ContextLayer exports a reasoning receipt (structured blocks) into the PR description so “what we tried / rejected / why this fix” doesn’t die in chat. Local-first, MCP for Cursor. Friends beta open — DM if you review AI PRs.

---

## Internal notes (don’t publish)

- Pair with mini eval post when ready (comprehension quiz, diff-only vs export).
- Follow-up post: ContextLayer + Graphify (repo structure vs reasoning).
- ICP: semi-technical founders, EMs, AI-heavy teams — not DIY senior SWE.

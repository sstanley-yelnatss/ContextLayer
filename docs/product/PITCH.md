# ContextLayer — pitch (read-off / reference)

**~2 minutes.** Updated 2026-06-07 · v1.2 shipped on `develop`

---

## One line

**Git tracks what changed. We track the auditable, reviewable reasoning path that produced the change.**

Short: **Git tracks what changed. We track why the AI-assisted process produced it.**

Category: **AI change governance** — PR reasoning, human-in-the-loop review for agent-assisted development.

**vs chat-log tools:** Others version chat or agent logs. We version reasoning — and ship it to the PR.

---

## Problem

Teams using AI to write code are shipping more pull requests, but reviewers still only see the **diff**. The reasoning — what was tried, what failed, what evidence supports the change — lives in private chat logs. That gap kills trust, slows review, and makes compliance a nightmare.

Same pattern outside eng: security research, product bets, strategic calls. People lose threads, retest dead paths, and can't explain *why* they're confident.

---

## Solution

**ContextLayer (what we ship today)** is a local-first **reasoning graph** for serious work. You open a **workspace** for a question you're working through. Each **block** on the timeline is one reasoning step: assumption (or hypothesis for security work), action, evidence, conclusion — any subset. A **hygiene panel** flags open loops, stale threads, and dead ends so unfinished thinking doesn't vanish into chat history.

You curate the story; the app tracks **how belief changed**, not just what you wrote down. For eng teams, the wedge is a **PR reasoning package**: pick the blocks that explain this change, export clean markdown, paste it into the PR. Reviewers get *why*, not just the diff. **MCP** hooks into Cursor so the agent can log blocks into the same database while you work — no copy-paste from chat.

The capture and governance lane sits underneath that curated layer. While you work with AI, a **recorder** captures the observable session: prompts, tool calls, files read, tests run. Not every message becomes history — we use **checkpoints** at decision moments, like squash before you open a PR. That raw trace is the audit trail compliance teams need: local-first, redactable, self-hostable.

On merge, that trace **links to the PR** on GitHub. A bot can attach the summary, the ContextLayer export, and a link to the trace. **Trace CI** runs the GitHub playbook on AI work: secrets scanned, redaction applied, rules enforced (policy-as-code, think `.gitllm/rules.yml`), “was a reasoning artifact attached?” Enterprise gets private repos, SSO, and an evidence pack reviewers and security can actually inspect.

**How the two lanes fit together:** the messy AI session gets captured and checkpointed. You (or MCP) distill it into structured ContextLayer blocks — the human-readable reasoning graph. You export the blocks for the PR. Reviewers read the curated story; compliance reads the raw trace if they need to. Same PR, two views: **structured graph for humans, observable trace for policy.**

We're not building another notes app, not logging every prompt, and not replacing GitHub. We're the **reasoning and governance layer** between AI-assisted work and human review.

---

## How it works (30 seconds)

1. **Work** in Cursor / your IDE with AI; log reasoning via MCP or the desktop app.
2. **Curate** blocks on a timeline; hygiene shows orphans, dead ends, open belief.
3. **Export** selected blocks as PR-ready markdown for reviewers.

Data stays local (`~/.contextlayer/graph.db`). Markdown exports are views, not the source of truth.

---

## Who it's for

| Audience | Pitch |
|----------|--------|
| **Agent DevOps** (primary) | Eng teams drowning in AI-assisted PRs — attach a **reasoning receipt** to every PR; assumption → action → evidence → conclusion |
| **Investigator** | Bug bounty, security research, debugging — structured investigations with **hypothesis** fields and hygiene for open loops |

---

## Full product vision

The Solution section above is the quick blurb. Detail:

| Layer | What | Status |
|-------|------|--------|
| **Curated reasoning** | ContextLayer workspaces, blocks, hygiene, PR export | **Shipping** (v1.2) |
| **Raw capture & governance** | Checkpointed AI session trace, GitHub PR link, policy CI, redaction | **Roadmap** (co-founder lane) |

**Agent Trace** (Cursor's format) = optional attribution plumbing later — line-level “this code came from that session.” **We** own structured reasoning, hygiene, and the PR narrative.

Analogy: Git is the format; GitHub is the business. Agent Trace is attribution plumbing; we are the reviewable reasoning platform.

---

## Complement, not compete

| Tool | They answer | We answer |
|------|-------------|-----------|
| **Graphify** | What the code is; what the PR touches | Why the author is confident in the change |
| **Linear** | What work exists; what shipped | The epistemic timeline behind the change |
| **Copilot Code Review** | Possible bugs in the diff | How the change was produced; paths rejected |

---

## What's shipped today (v1.2)

- Desktop app (Tauri + React): workspaces, block timeline, hygiene panel
- MCP server: log/edit/delete blocks, hygiene, **PR export** by id or title
- Local SQLite, shared DB between app and MCP
- CI on core crates + MCP

---

## What's next

- PR `develop` → `main` release; GitHub Releases
- Git-native committed artifact (optional file in repo)
- Raw trace capture + GitHub App (co-founder lane)
- Trace CI / policy rules (`.gitllm/rules.yml` style)

---

## Close

We're not another notes app or chat logger. We're the **reasoning layer** between AI-assisted work and human review — starting with the PR, expanding into capture and governance.

**Ask:** Dogfood partners who ship AI-heavy PRs and need reviewers to trust the process, not just the diff.

---

**Deep refs:** [POSITIONING.md](./POSITIONING.md) · [PRD-addendum-merged-vision.md](./PRD-addendum-merged-vision.md) · [MCP-TOOLS.md](./MCP-TOOLS.md)

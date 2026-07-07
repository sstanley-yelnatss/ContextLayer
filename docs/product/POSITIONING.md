# ContextLayer — positioning & copy

**Status:** Living doc for site, README, pitch decks, and co-founder alignment.  
**Last updated:** 2026-06-07

---

## Primary one-liner (merged product thesis)

Use this as the default hero line for site, GitHub org description, and investor one-pagers:

> **Git tracks what changed. We track the auditable, reviewable reasoning path that produced the change — raw trace for compliance, structured graph for humans.**

**Short variant (tighter hero):**

> **Git tracks what changed. We track why the AI-assisted process produced it.**

**Category label:**

AI change governance · PR reasoning · Human-in-the-loop review for agent-assisted development

---

## Dual pitch (keep both)

| Pitch | Audience | One line |
|-------|----------|----------|
| **Investigator** | Bug bounty, security research, debugging SWEs | Structured local investigations with hygiene for open loops |
| **Agent DevOps** | Eng teams drowning in AI-assisted PR volume | Reasoning receipt attached to a PR so reviewers see *why*, not just *what* changed |

---

## Competitive differentiation (vs chat-log tools)

Use on site, deck, and Product Hunt:

> **Others version chat or agent logs. We version reasoning — and ship it to the PR.**

| They (Twigg, OneContext, raw trace) | We (ContextLayer) |
|-------------------------------------|-------------------|
| Version **conversation** or session trees | Version **structured reasoning** (hypothesis → evidence → conclusion) |
| Research / compile context for the agent | **Hygiene + PR export** for human reviewers |
| Often cloud or session-centric | **Local-first** SQLite; MCP + desktop share one graph |

## Complement, not compete

| Tool | They answer | We answer |
|------|-------------|-----------|
| **Graphify** | What the *code* is; what a PR *touches* | Why the author is *confident* in the change |
| **Linear** | What *work* exists; what *shipped* | The epistemic timeline behind the change |
| **Copilot Code Review** | Possible bugs in the diff | How the change was produced; alternatives rejected |
| **Agent Trace** (format) | Which lines came from which AI session | Structured reasoning, hygiene, PR narrative, governance |
| **GCC** | Compile repo context for coding agents | Compile **reasoning** for reviewers; tiered MCP index |

---

## Agent Trace — relationship (not dependency)

**Agent Trace is a file format for code attribution** (file/line → conversation URL). It is explicitly **not** a product: no UI, no quality assessment, no governance, no structured reasoning.

We are **not** “Agent Trace with extra steps.” Our product is the **reasoning layer + PR workflow** Agent Trace does not define.

**Interoperability strategy (recommended):**

- **Core value:** ContextLayer workspaces, blocks, hygiene, PR markdown export.
- **Optional later:** Emit or link Agent Trace records so attribution tooling can find our reasoning artifact URL in `related` metadata.
- **Do not** let the format dictate product scope or positioning.

Analogy: Git is the format; GitHub is the business. Agent Trace is attribution plumbing; we are the reviewable reasoning platform.

---

## What we say publicly vs internally

| Say | Avoid |
|-----|-------|
| Policy-compliant AI development trace | “We save your full chat history” |
| Structured reasoning for PR review | “Git for LLM conversations” (consumer-ish) |
| Reasoning receipt / provenance | Hidden chain-of-thought |
| Checkpoints + curated blocks | Every prompt is a commit |

---

## Name (TBD)

Working options: **ContextLayer** (product), TraceForge, GitLLM. Name is not a blocker for execution; ship wedge first.

**“Umbrella + modules”** meant one company brand with named parts (e.g. ContextLayer = reasoning app; future capture/hosting module) — not required now.

---

## Changelog

| Date | Change |
|------|--------|
| 2026-06-07 | Initial positioning doc; merged one-liner; Agent Trace relationship |
| 2026-06-07 | Competitive one-liner vs Twigg/OneContext; GCC complement row |

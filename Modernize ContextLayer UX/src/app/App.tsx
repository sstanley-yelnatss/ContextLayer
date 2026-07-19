import { useState } from "react";
import {
  ChevronLeft,
  Plus,
  Zap,
  Download,
  Camera,
  MoreHorizontal,
  Circle,
  GitBranch,
} from "lucide-react";

type BlockType =
  | "hypothesis"
  | "action"
  | "evidence"
  | "conclusion"
  | "decision"
  | "assumption";
type BlockStatus = "open" | "confirmed" | "rejected" | "review";

interface Block {
  id: string;
  type: BlockType;
  status: BlockStatus;
  title: string;
  body: string;
  timestamp: string;
  tags: string[];
}

interface Workspace {
  id: string;
  name: string;
  description: string;
  openLoops: number;
  reasoningDebt: number;
  blocks: Block[];
}

const TYPE_META: Record<
  BlockType,
  { label: string; borderColor: string; textColor: string; dotColor: string }
> = {
  hypothesis: {
    label: "HYPOTHESIS",
    borderColor: "#fbbf24",
    textColor: "#fbbf24",
    dotColor: "#fbbf24",
  },
  action: {
    label: "ACTION",
    borderColor: "#38bdf8",
    textColor: "#38bdf8",
    dotColor: "#38bdf8",
  },
  evidence: {
    label: "EVIDENCE",
    borderColor: "#a78bfa",
    textColor: "#a78bfa",
    dotColor: "#a78bfa",
  },
  conclusion: {
    label: "CONCLUSION",
    borderColor: "#34d399",
    textColor: "#34d399",
    dotColor: "#34d399",
  },
  decision: {
    label: "DECISION",
    borderColor: "#fb923c",
    textColor: "#fb923c",
    dotColor: "#fb923c",
  },
  assumption: {
    label: "ASSUMPTION",
    borderColor: "#fb7185",
    textColor: "#fb7185",
    dotColor: "#fb7185",
  },
};

const STATUS_META: Record<
  BlockStatus,
  { label: string; bg: string; text: string; border: string; dot: string }
> = {
  open: {
    label: "OPEN",
    bg: "rgba(251,191,36,0.08)",
    text: "#fbbf24",
    border: "rgba(251,191,36,0.2)",
    dot: "#fbbf24",
  },
  confirmed: {
    label: "CONFIRMED",
    bg: "rgba(52,211,153,0.08)",
    text: "#34d399",
    border: "rgba(52,211,153,0.2)",
    dot: "#34d399",
  },
  rejected: {
    label: "REJECTED",
    bg: "rgba(248,113,113,0.08)",
    text: "#f87171",
    border: "rgba(248,113,113,0.2)",
    dot: "#f87171",
  },
  review: {
    label: "REVIEW",
    bg: "rgba(56,189,248,0.08)",
    text: "#38bdf8",
    border: "rgba(56,189,248,0.2)",
    dot: "#38bdf8",
  },
};

const WORKSPACES: Workspace[] = [
  {
    id: "session-store-race-fix",
    name: "session-store-race-fix",
    description: "Investigating race condition introduced by agent's mutex patch",
    openLoops: 2,
    reasoningDebt: 1,
    blocks: [
      {
        id: "b1",
        type: "hypothesis",
        status: "open",
        title: "Agent's patch masks the race condition instead of fixing it",
        body: "The session store was written under an assumption of single-threaded access. The agent's mutex addition may serialize the wrong critical section — protecting the read but not the read-modify-write sequence.",
        timestamp: "2026-07-13T14:22:00",
        tags: ["agent", "concurrency"],
      },
      {
        id: "b2",
        type: "action",
        status: "confirmed",
        title: "Had the agent write a failing test before applying the patch",
        body: "Prompted Claude to generate a reproducer that reliably triggers the race. Ran it 100× on main — failed 34 times. This gives us a baseline to validate the fix against.",
        timestamp: "2026-07-13T14:31:00",
        tags: ["testing"],
      },
      {
        id: "b3",
        type: "evidence",
        status: "confirmed",
        title: "Test fails on main, passes with patch. Race was real",
        body: "go test -race -count=100 shows 0 failures with the patch applied. The reproducer confirms the condition was genuine and the fix is effective.",
        timestamp: "2026-07-13T14:45:00",
        tags: ["evidence", "test-results"],
      },
      {
        id: "b4",
        type: "conclusion",
        status: "confirmed",
        title: "Patch accepted. Rationale exported to the PR",
        body: "The mutex placement is correct. Race is on the read-modify-write of the session map, not the write path alone. Exported full reasoning chain to PR #2241.",
        timestamp: "2026-07-13T15:02:00",
        tags: ["decision", "exported"],
      },
    ],
  },
  {
    id: "auth-provider-migration",
    name: "auth-provider-migra...",
    description: "NextAuth v5 migration — provider config and session shape audit",
    openLoops: 0,
    reasoningDebt: 3,
    blocks: [
      {
        id: "c1",
        type: "assumption",
        status: "open",
        title: "NextAuth v5 breaking changes are confined to the adapter layer",
        body: "Migration guide implies session shape is preserved. JWT token structure may have changed in the v5 beta — needs verification before we trust existing session reads.",
        timestamp: "2026-07-12T10:15:00",
        tags: ["auth", "migration"],
      },
      {
        id: "c2",
        type: "hypothesis",
        status: "open",
        title: "Google OAuth callback URL mismatch causing 401 on redirect",
        body: "Console errors suggest the redirect URI registered in Google Cloud Console no longer matches after the domain change. The agent updated the env var but may not have updated the OAuth app config.",
        timestamp: "2026-07-12T11:03:00",
        tags: ["oauth", "google"],
      },
      {
        id: "c3",
        type: "action",
        status: "review",
        title: "Asked agent to audit all OAuth callback URLs across environments",
        body: "Claude enumerated dev, staging, and prod callback URLs. Found 2 mismatches. Awaiting confirmation that the Google Cloud Console entries were updated.",
        timestamp: "2026-07-12T11:45:00",
        tags: ["audit", "oauth"],
      },
    ],
  },
  {
    id: "flaky-ci-timeouts",
    name: "flaky-ci-timeouts",
    description: "Root-cause analysis for CI timeout cluster in nightly runs",
    openLoops: 1,
    reasoningDebt: 0,
    blocks: [
      {
        id: "d1",
        type: "evidence",
        status: "review",
        title: "Timeouts cluster between 03:00–04:00 UTC consistently",
        body: "14-day analysis of CI logs shows 91% of timeout failures fall in a 60-minute window. Correlates with scheduled GitHub Actions runner maintenance windows.",
        timestamp: "2026-07-11T09:00:00",
        tags: ["ci", "flaky", "timing"],
      },
      {
        id: "d2",
        type: "decision",
        status: "open",
        title: "Reschedule nightly runs to 05:00 UTC or add retry logic",
        body: "Two options: shift the cron expression, or add a 2-retry wrapper on the test suite step. Retry adds 8 min worst-case overhead per run but is more resilient.",
        timestamp: "2026-07-11T09:22:00",
        tags: ["ci", "decision"],
      },
    ],
  },
];

function formatTimestamp(iso: string) {
  const d = new Date(iso);
  return (
    d.toLocaleDateString("en-US", { month: "short", day: "numeric" }) +
    " · " +
    d.toLocaleTimeString("en-US", {
      hour: "2-digit",
      minute: "2-digit",
      hour12: false,
    })
  );
}

type FilterType = BlockType | "all";

const FILTER_OPTIONS: { value: FilterType; label: string }[] = [
  { value: "all", label: "All" },
  { value: "hypothesis", label: "Hypothesis" },
  { value: "action", label: "Action" },
  { value: "evidence", label: "Evidence" },
  { value: "conclusion", label: "Conclusion" },
  { value: "decision", label: "Decision" },
  { value: "assumption", label: "Assumption" },
];

export default function App() {
  const [activeId, setActiveId] = useState(WORKSPACES[0].id);
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [filter, setFilter] = useState<FilterType>("all");

  const workspace = WORKSPACES.find((w) => w.id === activeId)!;
  const blocks =
    filter === "all"
      ? workspace.blocks
      : workspace.blocks.filter((b) => b.type === filter);

  const health = {
    total: workspace.blocks.length,
    open: workspace.blocks.filter((b) => b.status === "open").length,
    confirmed: workspace.blocks.filter((b) => b.status === "confirmed").length,
    rejected: workspace.blocks.filter((b) => b.status === "rejected").length,
    review: workspace.blocks.filter((b) => b.status === "review").length,
    debt: workspace.reasoningDebt,
    orphans: 0,
    stale: 0,
    openLoops: workspace.openLoops,
    deadEnds: workspace.blocks.filter((b) => b.status === "rejected").length,
    decisions: workspace.blocks.filter(
      (b) => b.type === "decision" || b.type === "conclusion"
    ).length,
  };

  return (
    <div
      className="flex h-screen bg-background text-foreground overflow-hidden"
      style={{ fontFamily: "'Inter', system-ui, sans-serif" }}
    >
      {/* ── LEFT SIDEBAR ────────────────────────────────────── */}
      <aside
        className="w-52 flex-shrink-0 flex flex-col overflow-hidden"
        style={{
          background: "#0f0f12",
          borderRight: "1px solid rgba(255,255,255,0.07)",
        }}
      >
        {/* Logo */}
        <div
          className="flex items-center gap-2.5 px-4 py-[14px]"
          style={{ borderBottom: "1px solid rgba(255,255,255,0.07)" }}
        >
          <div
            className="w-[18px] h-[18px] rounded-[3px] flex items-center justify-center flex-shrink-0"
            style={{ background: "rgba(34,211,238,0.15)" }}
          >
            <div
              className="w-[9px] h-[9px] rounded-[2px]"
              style={{ background: "#22d3ee" }}
            />
          </div>
          <span
            className="text-[13px] font-semibold tracking-tight"
            style={{ color: "#e2e2e7" }}
          >
            ContextLayer
          </span>
        </div>

        {/* Workspace list */}
        <div className="flex-1 overflow-y-auto px-2 pt-4 pb-2">
          <p
            className="px-2 mb-1.5 text-[10px] font-medium tracking-widest uppercase"
            style={{
              fontFamily: "'JetBrains Mono', monospace",
              color: "#5c5c6e",
            }}
          >
            Workspaces
          </p>
          <div className="space-y-[2px]">
            {WORKSPACES.map((ws) => {
              const active = ws.id === activeId;
              return (
                <button
                  key={ws.id}
                  onClick={() => {
                    setActiveId(ws.id);
                    setFilter("all");
                    setExpandedId(null);
                  }}
                  className="w-full text-left px-2 py-[6px] rounded-[3px] text-[12.5px] transition-colors truncate block"
                  style={{
                    fontFamily: "'JetBrains Mono', monospace",
                    background: active ? "rgba(34,211,238,0.08)" : "transparent",
                    color: active ? "#22d3ee" : "#5c5c6e",
                  }}
                  onMouseEnter={(e) => {
                    if (!active)
                      (e.currentTarget as HTMLButtonElement).style.color =
                        "#e2e2e7";
                  }}
                  onMouseLeave={(e) => {
                    if (!active)
                      (e.currentTarget as HTMLButtonElement).style.color =
                        "#5c5c6e";
                  }}
                >
                  {ws.name}
                </button>
              );
            })}
          </div>

          {/* Hygiene */}
          <p
            className="px-2 mt-6 mb-1.5 text-[10px] font-medium tracking-widest uppercase"
            style={{
              fontFamily: "'JetBrains Mono', monospace",
              color: "#5c5c6e",
            }}
          >
            Hygiene
          </p>
          <div className="space-y-[3px] px-1">
            {WORKSPACES.filter(
              (ws) => ws.openLoops > 0 || ws.reasoningDebt > 0
            ).map((ws) => (
              <div
                key={ws.id}
                style={{ fontFamily: "'JetBrains Mono', monospace" }}
              >
                {ws.openLoops > 0 && (
                  <div className="flex items-center gap-1.5 py-[3px]">
                    <span
                      className="w-[6px] h-[6px] rounded-full flex-shrink-0"
                      style={{ background: "#fbbf24" }}
                    />
                    <span className="text-[11px]" style={{ color: "#fbbf24" }}>
                      {ws.openLoops} open loop
                      {ws.openLoops !== 1 ? "s" : ""}
                    </span>
                  </div>
                )}
                {ws.reasoningDebt > 0 && (
                  <div className="flex items-center gap-1.5 py-[3px]">
                    <span
                      className="w-[6px] h-[6px] rounded-full flex-shrink-0"
                      style={{ background: "#fb7185" }}
                    />
                    <span className="text-[11px]" style={{ color: "#fb7185" }}>
                      {ws.reasoningDebt} reasoning debt
                    </span>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>

        {/* New workspace */}
        <div
          className="px-3 py-3"
          style={{ borderTop: "1px solid rgba(255,255,255,0.07)" }}
        >
          <button
            className="w-full flex items-center gap-2 px-2 py-[6px] rounded-[3px] text-[12px] transition-colors"
            style={{ color: "#5c5c6e" }}
            onMouseEnter={(e) =>
              ((e.currentTarget as HTMLButtonElement).style.color = "#e2e2e7")
            }
            onMouseLeave={(e) =>
              ((e.currentTarget as HTMLButtonElement).style.color = "#5c5c6e")
            }
          >
            <Plus size={12} />
            <span>New workspace</span>
          </button>
        </div>
      </aside>

      {/* ── MAIN CONTENT ────────────────────────────────────── */}
      <main className="flex-1 min-w-0 flex flex-col overflow-hidden">
        {/* Header */}
        <div
          className="px-6 pt-5 pb-4 flex-shrink-0"
          style={{ borderBottom: "1px solid rgba(255,255,255,0.07)" }}
        >
          <div className="flex items-start justify-between gap-4">
            <div className="min-w-0">
              <button
                className="flex items-center gap-1 text-[11px] mb-1 transition-colors"
                style={{
                  fontFamily: "'JetBrains Mono', monospace",
                  color: "#5c5c6e",
                }}
                onMouseEnter={(e) =>
                  ((e.currentTarget as HTMLButtonElement).style.color =
                    "#e2e2e7")
                }
                onMouseLeave={(e) =>
                  ((e.currentTarget as HTMLButtonElement).style.color =
                    "#5c5c6e")
                }
              >
                <ChevronLeft size={11} />
                Workspaces
              </button>
              <h1
                className="text-[17px] font-semibold tracking-tight truncate"
                style={{
                  fontFamily: "'JetBrains Mono', monospace",
                  color: "#e2e2e7",
                }}
              >
                {workspace.name}
              </h1>
              <p
                className="text-[12px] mt-0.5 truncate"
                style={{ color: "#5c5c6e" }}
              >
                {workspace.description}
              </p>
            </div>

            <div className="flex items-center gap-2 flex-shrink-0 mt-1">
              <button
                className="flex items-center gap-1.5 px-3 py-[6px] rounded-[3px] text-[11.5px] font-medium transition-colors"
                style={{
                  background: "rgba(255,255,255,0.05)",
                  border: "1px solid rgba(255,255,255,0.09)",
                  color: "#9898a8",
                }}
                onMouseEnter={(e) => {
                  (e.currentTarget as HTMLButtonElement).style.background =
                    "rgba(255,255,255,0.08)";
                  (e.currentTarget as HTMLButtonElement).style.color =
                    "#e2e2e7";
                }}
                onMouseLeave={(e) => {
                  (e.currentTarget as HTMLButtonElement).style.background =
                    "rgba(255,255,255,0.05)";
                  (e.currentTarget as HTMLButtonElement).style.color =
                    "#9898a8";
                }}
              >
                <Camera size={11} />
                Checkpoint
              </button>
              <button
                className="flex items-center gap-1.5 px-3 py-[6px] rounded-[3px] text-[11.5px] font-medium transition-colors"
                style={{
                  background: "rgba(34,211,238,0.08)",
                  border: "1px solid rgba(34,211,238,0.2)",
                  color: "#22d3ee",
                }}
                onMouseEnter={(e) => {
                  (e.currentTarget as HTMLButtonElement).style.background =
                    "rgba(34,211,238,0.14)";
                }}
                onMouseLeave={(e) => {
                  (e.currentTarget as HTMLButtonElement).style.background =
                    "rgba(34,211,238,0.08)";
                }}
              >
                <Zap size={11} />
                Start Capture
              </button>
              <button
                className="flex items-center gap-1.5 px-3 py-[6px] rounded-[3px] text-[11.5px] font-medium transition-colors"
                style={{
                  background: "rgba(52,211,153,0.08)",
                  border: "1px solid rgba(52,211,153,0.2)",
                  color: "#34d399",
                }}
                onMouseEnter={(e) => {
                  (e.currentTarget as HTMLButtonElement).style.background =
                    "rgba(52,211,153,0.14)";
                }}
                onMouseLeave={(e) => {
                  (e.currentTarget as HTMLButtonElement).style.background =
                    "rgba(52,211,153,0.08)";
                }}
              >
                <Download size={11} />
                Export PR
              </button>
            </div>
          </div>

          {/* Filter pills */}
          <div className="flex items-center gap-1 mt-4">
            <div
              className="flex items-center gap-[2px] p-[3px] rounded-[4px]"
              style={{ background: "rgba(255,255,255,0.04)", border: "1px solid rgba(255,255,255,0.07)" }}
            >
              {FILTER_OPTIONS.map(({ value, label }) => {
                const active = filter === value;
                return (
                  <button
                    key={value}
                    onClick={() => setFilter(value)}
                    className="px-2.5 py-[4px] rounded-[3px] text-[11px] font-medium transition-colors"
                    style={{
                      fontFamily: "'JetBrains Mono', monospace",
                      background: active ? "rgba(255,255,255,0.09)" : "transparent",
                      color: active
                        ? "#e2e2e7"
                        : value !== "all" && !active
                        ? TYPE_META[value as BlockType]?.textColor
                        : "#5c5c6e",
                      opacity: active ? 1 : value !== "all" ? 0.65 : 1,
                    }}
                  >
                    {label}
                  </button>
                );
              })}
            </div>
          </div>
        </div>

        {/* Block feed */}
        <div className="flex-1 overflow-y-auto px-6 py-5">
          {/* Add block */}
          <button
            className="w-full flex items-center gap-2 px-3 py-[9px] rounded-[3px] text-[12px] mb-3 transition-colors"
            style={{
              color: "#5c5c6e",
              border: "1px dashed rgba(255,255,255,0.1)",
            }}
            onMouseEnter={(e) => {
              (e.currentTarget as HTMLButtonElement).style.color = "#9898a8";
              (e.currentTarget as HTMLButtonElement).style.borderColor =
                "rgba(255,255,255,0.18)";
            }}
            onMouseLeave={(e) => {
              (e.currentTarget as HTMLButtonElement).style.color = "#5c5c6e";
              (e.currentTarget as HTMLButtonElement).style.borderColor =
                "rgba(255,255,255,0.1)";
            }}
          >
            <Plus size={12} />
            <span>Add block</span>
          </button>

          <div className="space-y-2">
            {blocks.map((block) => {
              const t = TYPE_META[block.type];
              const s = STATUS_META[block.status];
              const expanded = expandedId === block.id;

              return (
                <div
                  key={block.id}
                  onClick={() => setExpandedId(expanded ? null : block.id)}
                  className="rounded-[4px] cursor-pointer transition-colors"
                  style={{
                    background: "#111115",
                    border: "1px solid rgba(255,255,255,0.07)",
                    borderLeft: `2px solid ${t.borderColor}`,
                  }}
                  onMouseEnter={(e) => {
                    (e.currentTarget as HTMLDivElement).style.background =
                      "#161619";
                  }}
                  onMouseLeave={(e) => {
                    (e.currentTarget as HTMLDivElement).style.background =
                      "#111115";
                  }}
                >
                  <div className="px-4 py-3">
                    {/* Type + status row */}
                    <div className="flex items-center justify-between mb-2">
                      <div className="flex items-center gap-2.5">
                        <span
                          className="text-[10px] font-medium tracking-widest"
                          style={{
                            fontFamily: "'JetBrains Mono', monospace",
                            color: t.textColor,
                          }}
                        >
                          {t.label}
                        </span>
                        <span
                          className="inline-flex items-center gap-1 px-1.5 py-[3px] rounded-[3px] text-[10px] font-medium"
                          style={{
                            fontFamily: "'JetBrains Mono', monospace",
                            background: s.bg,
                            color: s.text,
                            border: `1px solid ${s.border}`,
                          }}
                        >
                          <span
                            className="w-[5px] h-[5px] rounded-full"
                            style={{ background: s.dot }}
                          />
                          {s.label}
                        </span>
                      </div>
                      <div className="flex items-center gap-2.5">
                        <span
                          className="text-[11px]"
                          style={{
                            fontFamily: "'JetBrains Mono', monospace",
                            color: "#5c5c6e",
                          }}
                        >
                          {formatTimestamp(block.timestamp)}
                        </span>
                        <button
                          onClick={(e) => e.stopPropagation()}
                          style={{ color: "#5c5c6e" }}
                          onMouseEnter={(e) =>
                            ((e.currentTarget as HTMLButtonElement).style.color =
                              "#e2e2e7")
                          }
                          onMouseLeave={(e) =>
                            ((e.currentTarget as HTMLButtonElement).style.color =
                              "#5c5c6e")
                          }
                        >
                          <MoreHorizontal size={14} />
                        </button>
                      </div>
                    </div>

                    {/* Title */}
                    <p
                      className="text-[13.5px] font-medium leading-snug"
                      style={{ color: "#e2e2e7" }}
                    >
                      {block.title}
                    </p>

                    {/* Expanded body */}
                    {expanded && (
                      <p
                        className="mt-2 text-[12.5px] leading-relaxed"
                        style={{ color: "#7a7a8c" }}
                      >
                        {block.body}
                      </p>
                    )}

                    {/* Tags */}
                    {block.tags.length > 0 && (
                      <div className="flex flex-wrap gap-1.5 mt-2.5">
                        {block.tags.map((tag) => (
                          <span
                            key={tag}
                            className="px-[7px] py-[2px] rounded-[3px] text-[10.5px]"
                            style={{
                              fontFamily: "'JetBrains Mono', monospace",
                              background: "rgba(255,255,255,0.04)",
                              color: "#5c5c6e",
                              border: "1px solid rgba(255,255,255,0.07)",
                            }}
                          >
                            {tag}
                          </span>
                        ))}
                      </div>
                    )}
                  </div>
                </div>
              );
            })}
          </div>

          {blocks.length === 0 && (
            <div className="flex flex-col items-center justify-center py-16 gap-2">
              <Circle size={20} style={{ color: "#2a2a33" }} />
              <p className="text-[12px]" style={{ color: "#5c5c6e" }}>
                No {filter !== "all" ? filter : ""} blocks in this workspace
              </p>
            </div>
          )}
        </div>
      </main>

      {/* ── RIGHT PANEL — HEALTH ────────────────────────────── */}
      <aside
        className="w-[236px] flex-shrink-0 flex flex-col overflow-hidden"
        style={{
          background: "#0f0f12",
          borderLeft: "1px solid rgba(255,255,255,0.07)",
        }}
      >
        <div
          className="px-4 py-[14px]"
          style={{ borderBottom: "1px solid rgba(255,255,255,0.07)" }}
        >
          <p
            className="text-[10px] font-medium tracking-widest uppercase"
            style={{
              fontFamily: "'JetBrains Mono', monospace",
              color: "#5c5c6e",
            }}
          >
            Workspace Health
          </p>
        </div>

        <div className="flex-1 overflow-y-auto px-4 py-4">
          {/* Stat grid 2×3 */}
          <div className="grid grid-cols-2 gap-1.5 mb-5">
            {[
              { label: "Blocks", value: health.total, color: "#e2e2e7" },
              {
                label: "Open",
                value: health.open,
                color: health.open > 0 ? "#fbbf24" : "#e2e2e7",
              },
              {
                label: "Confirmed",
                value: health.confirmed,
                color: health.confirmed > 0 ? "#34d399" : "#e2e2e7",
              },
              {
                label: "Rejected",
                value: health.rejected,
                color: health.rejected > 0 ? "#f87171" : "#e2e2e7",
              },
              {
                label: "Needs Review",
                value: health.review,
                color: health.review > 0 ? "#38bdf8" : "#e2e2e7",
              },
              {
                label: "Reasoning Debt",
                value: health.debt,
                color: health.debt > 0 ? "#fb7185" : "#e2e2e7",
              },
            ].map(({ label, value, color }) => (
              <div
                key={label}
                className="rounded-[3px] px-3 py-[10px]"
                style={{
                  background: "#111115",
                  border: "1px solid rgba(255,255,255,0.07)",
                }}
              >
                <p
                  className="text-[9.5px] font-medium mb-1.5 leading-tight"
                  style={{
                    fontFamily: "'JetBrains Mono', monospace",
                    color: "#5c5c6e",
                  }}
                >
                  {label}
                </p>
                <p
                  className="text-[22px] font-semibold leading-none"
                  style={{ color }}
                >
                  {value}
                </p>
              </div>
            ))}
          </div>

          {/* Detail list */}
          <div>
            {[
              { label: "Orphans", value: health.orphans, warn: false, pos: false },
              { label: "Stale", value: health.stale, warn: false, pos: false },
              { label: "Still Open", value: health.openLoops, warn: true, pos: false },
              { label: "Dead Ends", value: health.deadEnds, warn: false, pos: false },
              { label: "Decisions", value: health.decisions, warn: false, pos: true },
            ].map(({ label, value, warn, pos }) => (
              <div
                key={label}
                className="flex items-center justify-between py-[9px]"
                style={{ borderBottom: "1px solid rgba(255,255,255,0.05)" }}
              >
                <span className="text-[12px]" style={{ color: "#7a7a8c" }}>
                  {label}
                </span>
                <span
                  className="text-[12px] font-medium"
                  style={{
                    fontFamily: "'JetBrains Mono', monospace",
                    color:
                      warn && value > 0
                        ? "#fbbf24"
                        : pos && value > 0
                        ? "#34d399"
                        : "#e2e2e7",
                  }}
                >
                  {value}
                </span>
              </div>
            ))}
          </div>

          {/* Block type legend */}
          <div className="mt-5">
            <p
              className="text-[10px] font-medium tracking-widest uppercase mb-2.5"
              style={{
                fontFamily: "'JetBrains Mono', monospace",
                color: "#5c5c6e",
              }}
            >
              Block Types
            </p>
            <div className="space-y-[7px]">
              {Object.entries(TYPE_META).map(([type, meta]) => (
                <div key={type} className="flex items-center gap-2">
                  <span
                    className="w-[8px] h-[8px] rounded-[2px] flex-shrink-0"
                    style={{ background: meta.dotColor }}
                  />
                  <span
                    className="text-[11px]"
                    style={{
                      fontFamily: "'JetBrains Mono', monospace",
                      color: meta.textColor,
                      opacity: 0.75,
                    }}
                  >
                    {meta.label}
                  </span>
                </div>
              ))}
            </div>
          </div>

          {/* Branch indicator */}
          <div
            className="mt-5 flex items-center gap-1.5 px-2.5 py-[8px] rounded-[3px]"
            style={{
              background: "rgba(255,255,255,0.03)",
              border: "1px solid rgba(255,255,255,0.06)",
            }}
          >
            <GitBranch size={11} style={{ color: "#5c5c6e" }} />
            <span
              className="text-[11px] truncate"
              style={{
                fontFamily: "'JetBrains Mono', monospace",
                color: "#5c5c6e",
              }}
            >
              {workspace.id}
            </span>
          </div>
        </div>
      </aside>
    </div>
  );
}

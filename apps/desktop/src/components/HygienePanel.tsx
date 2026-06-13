import type { HygieneItem, WorkspaceHygieneReport } from "../types";

interface Props {
  report: WorkspaceHygieneReport | null;
  loading: boolean;
  activeCategory: string | null;
  onSelectCategory: (category: string | null) => void;
  onSelectBlock: (blockId: string) => void;
}

const CATEGORIES = [
  { id: "orphans", label: "Orphans", key: "orphans" as const },
  { id: "stale", label: "Stale", key: "stale" as const },
  { id: "still_open", label: "Still open", key: "still_open" as const },
  { id: "dead_ends", label: "Dead ends", key: "dead_ends" as const },
  { id: "decisions", label: "Decisions", key: "decisions" as const },
];

export default function HygienePanel({
  report,
  loading,
  activeCategory,
  onSelectCategory,
  onSelectBlock,
}: Props) {
  if (loading) {
    return (
      <aside className="w-72 shrink-0 border-l border-zinc-800 bg-zinc-900/50 p-4">
        <p className="text-sm text-zinc-500">Loading health…</p>
      </aside>
    );
  }

  if (!report) return null;

  const s = report.summary;

  function itemsFor(key: (typeof CATEGORIES)[number]["key"]): HygieneItem[] {
    return report![key];
  }

  const activeItems = activeCategory
    ? itemsFor(activeCategory as (typeof CATEGORIES)[number]["key"])
    : [];

  return (
    <aside className="w-72 shrink-0 overflow-y-auto border-l border-zinc-800 bg-zinc-900/50 p-4 max-h-screen">
      <h2 className="mb-3 text-sm font-medium text-zinc-200">Workspace health</h2>

      <div className="mb-4 grid grid-cols-2 gap-2 text-xs">
        <Stat label="Blocks" value={s.total_blocks} />
        <Stat label="Open" value={s.belief_open} />
        <Stat label="Confirmed" value={s.belief_confirmed} />
        <Stat label="Rejected" value={s.belief_rejected} />
        <Stat label="Needs review" value={s.needs_review} accent="sky" />
        <Stat label="Reasoning debt" value={s.reasoning_debt} accent="orange" />
      </div>

      <div className="mb-4 space-y-1">
        {CATEGORIES.map((cat) => {
          const count = itemsFor(cat.key).length;
          const active = activeCategory === cat.key;
          return (
            <button
              key={cat.key}
              type="button"
              onClick={() => onSelectCategory(active ? null : cat.key)}
              className={`flex w-full cursor-pointer items-center justify-between rounded-lg px-3 py-2 text-left text-xs transition ${
                active
                  ? "bg-zinc-800 text-zinc-100"
                  : "text-zinc-400 hover:bg-zinc-800/60 hover:text-zinc-200"
              }`}
            >
              <span>{cat.label}</span>
              <span
                className={`rounded-full px-2 py-0.5 ${
                  count > 0 ? "bg-orange-950/60 text-orange-300" : "text-zinc-600"
                }`}
              >
                {count}
              </span>
            </button>
          );
        })}
      </div>

      {activeCategory && (
        <div className="space-y-2 border-t border-zinc-800 pt-3">
          <p className="text-xs font-medium uppercase tracking-wide text-zinc-500">
            {CATEGORIES.find((c) => c.key === activeCategory)?.label}
          </p>
          {activeItems.length === 0 ? (
            <p className="text-xs text-zinc-600">Nothing here — nice.</p>
          ) : (
            activeItems.map((item) => (
              <button
                key={`${item.category}-${item.block_id}`}
                type="button"
                onClick={() => onSelectBlock(item.block_id)}
                className="w-full cursor-pointer rounded-lg border border-zinc-800 bg-zinc-950/50 px-3 py-2 text-left hover:border-zinc-600"
              >
                <p className="text-xs text-orange-300/90">{item.message}</p>
                <p className="mt-1 text-xs text-zinc-400 line-clamp-2">{item.preview}</p>
                {item.days_open != null && item.days_open > 0 && (
                  <p className="mt-1 text-[10px] text-zinc-600">{item.days_open}d</p>
                )}
              </button>
            ))
          )}
        </div>
      )}
    </aside>
  );
}

function Stat({
  label,
  value,
  accent,
}: {
  label: string;
  value: number;
  accent?: "sky" | "orange";
}) {
  return (
    <div className="rounded-lg border border-zinc-800 bg-zinc-950/40 px-2 py-1.5">
      <p className="text-zinc-500">{label}</p>
      <p
        className={`text-base font-medium ${
          accent === "sky"
            ? "text-sky-300"
            : accent === "orange"
              ? "text-orange-300"
              : "text-zinc-100"
        }`}
      >
        {value}
      </p>
    </div>
  );
}

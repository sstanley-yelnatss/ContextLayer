import { HYGIENE_CATEGORIES, type HygieneItem, type WorkspaceHygieneReport } from "../types";

interface Props {
  report: WorkspaceHygieneReport | null;
  loading: boolean;
  activeCategory: string | null;
  onSelectCategory: (category: string | null) => void;
  onSelectBlock: (blockId: string) => void;
}

export default function HygienePanel({
  report,
  loading,
  activeCategory,
  onSelectCategory,
  onSelectBlock,
}: Props) {
  if (loading) {
    return (
      <aside className="w-80 shrink-0 border-l border-zinc-800 bg-zinc-900/50 p-5">
        <p className="text-sm text-zinc-500">Loading health…</p>
      </aside>
    );
  }

  if (!report) return null;

  const s = report.summary;

  function itemsFor(key: (typeof HYGIENE_CATEGORIES)[number]["key"]): HygieneItem[] {
    return report![key];
  }

  const activeItems = activeCategory
    ? itemsFor(activeCategory as (typeof HYGIENE_CATEGORIES)[number]["key"])
    : [];

  return (
    <aside className="w-80 shrink-0 overflow-y-auto border-l border-zinc-800 bg-zinc-900/50 p-5 max-h-screen">
      <h2 className="mb-4 text-base font-medium text-zinc-200">Workspace health</h2>

      <div className="mb-5 grid grid-cols-2 gap-2.5 text-sm">
        <Stat label="Blocks" value={s.total_blocks} />
        <Stat label="Open" value={s.belief_open} />
        <Stat label="Confirmed" value={s.belief_confirmed} />
        <Stat label="Rejected" value={s.belief_rejected} />
        <Stat label="Needs review" value={s.needs_review} accent="red" />
        <Stat label="Reasoning debt" value={s.reasoning_debt} accent="orange" />
      </div>

      <div className="mb-5 space-y-1.5">
        {HYGIENE_CATEGORIES.map((cat) => {
          const count = itemsFor(cat.key).length;
          const active = activeCategory === cat.key;
          return (
            <button
              key={cat.key}
              type="button"
              onClick={() => onSelectCategory(active ? null : cat.key)}
              className={`flex w-full cursor-pointer items-center justify-between rounded-lg px-3 py-2.5 text-left text-sm transition ${
                active
                  ? "bg-zinc-800 text-zinc-100"
                  : "text-zinc-400 hover:bg-zinc-800/60 hover:text-zinc-200"
              }`}
            >
              <span>{cat.label}</span>
              <span
                className={`rounded-full px-2.5 py-0.5 text-sm ${
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
        <div className="space-y-2.5 border-t border-zinc-800 pt-4">
          <p className="text-sm font-medium uppercase tracking-wide text-zinc-500">
            {HYGIENE_CATEGORIES.find((c) => c.key === activeCategory)?.label}
          </p>
          {activeItems.length === 0 ? (
            <p className="text-sm text-zinc-600">Nothing here. Nice.</p>
          ) : (
            activeItems.map((item) => (
              <button
                key={`${item.category}-${item.block_id}`}
                type="button"
                onClick={() => onSelectBlock(item.block_id)}
                className="w-full cursor-pointer rounded-lg border border-zinc-800 bg-zinc-950/50 px-3 py-2.5 text-left hover:border-zinc-600"
              >
                <p className="text-sm text-orange-300/90">{item.message}</p>
                <p className="mt-1 text-sm text-zinc-400 line-clamp-2">{item.preview}</p>
                {item.days_open != null && item.days_open > 0 && (
                  <p className="mt-1 text-xs text-zinc-600">{item.days_open}d</p>
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
  accent?: "red" | "orange";
}) {
  return (
    <div className="rounded-lg border border-zinc-800 bg-zinc-950/40 px-3 py-2">
      <p className="text-sm text-zinc-500">{label}</p>
      <p
        className={`text-lg font-medium ${
          accent === "red"
            ? "text-red-400"
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

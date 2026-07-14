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
      <aside className="w-72 shrink-0 border-l border-border bg-card/50 p-5">
        <p className="text-sm text-muted-foreground">Loading health…</p>
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
    <aside className="max-h-full w-72 shrink-0 overflow-y-auto border-l border-border bg-card/50 p-5">
      <h2 className="mb-4 text-[13px] font-semibold uppercase tracking-wide text-muted-foreground">
        Workspace health
      </h2>

      <div className="mb-5 grid grid-cols-2 gap-2">
        <Stat label="Blocks" value={s.total_blocks} />
        <Stat label="Open" value={s.belief_open} />
        <Stat label="Confirmed" value={s.belief_confirmed} />
        <Stat label="Rejected" value={s.belief_rejected} />
        <Stat label="Needs review" value={s.needs_review} accent="warn" />
        <Stat label="Reasoning debt" value={s.reasoning_debt} accent="warn" />
      </div>

      <div className="mb-5 space-y-0.5">
        {HYGIENE_CATEGORIES.map((cat) => {
          const count = itemsFor(cat.key).length;
          const active = activeCategory === cat.key;
          return (
            <button
              key={cat.key}
              type="button"
              onClick={() => onSelectCategory(active ? null : cat.key)}
              className={`flex w-full items-center justify-between rounded-[3px] px-2.5 py-2 text-left text-[13px] transition-colors ${
                active
                  ? "bg-[rgba(255,255,255,0.08)] text-foreground"
                  : "text-muted-foreground hover:bg-[rgba(255,255,255,0.04)] hover:text-foreground"
              }`}
            >
              <span>{cat.label}</span>
              <span
                className="font-mono-ui min-w-[1.75rem] rounded-[3px] px-2.5 py-1 text-center text-[13px] font-semibold tabular-nums"
                style={
                  count > 0
                    ? {
                        background: "rgba(251, 113, 133, 0.08)",
                        color: "var(--hygiene-warn)",
                      }
                    : { color: "var(--muted-foreground)" }
                }
              >
                {count}
              </span>
            </button>
          );
        })}
      </div>

      {activeCategory && (
        <div className="space-y-2 border-t border-border pt-4">
          <p className="text-[12px] font-medium uppercase tracking-wide text-muted-foreground">
            {HYGIENE_CATEGORIES.find((c) => c.key === activeCategory)?.label}
          </p>
          {activeItems.length === 0 ? (
            <p className="text-sm text-muted-foreground">Nothing here. Nice.</p>
          ) : (
            activeItems.map((item) => (
              <button
                key={`${item.category}-${item.block_id}`}
                type="button"
                onClick={() => onSelectBlock(item.block_id)}
                className="cl-surface-card w-full px-3 py-2.5 text-left transition-colors hover:bg-[#161619]"
              >
                <p className="text-sm" style={{ color: "var(--hygiene-warn)" }}>
                  {item.message}
                </p>
                <p className="mt-1 line-clamp-2 text-sm text-muted-foreground">{item.preview}</p>
                {item.days_open != null && item.days_open > 0 && (
                  <p className="mt-1 text-xs text-muted-foreground/80">
                    {item.days_open}d
                  </p>
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
  accent?: "warn";
}) {
  return (
    <div className="cl-surface-card px-3 py-2">
      <p className="text-xs text-muted-foreground">{label}</p>
      <p
        className="font-mono-ui text-lg font-semibold tabular-nums"
        style={accent === "warn" && value > 0 ? { color: "var(--hygiene-warn)" } : undefined}
      >
        {value}
      </p>
    </div>
  );
}

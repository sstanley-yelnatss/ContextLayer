import { useMemo, useState } from "react";
import { Zap } from "lucide-react";
import type { SessionGraph, SessionGraphLane, SessionGraphRow } from "../types";

const ROW_H = 56;
const LANE_W = 28;
const GRAPH_PAD_TOP = 12;

const FORK_COLORS = ["#a78bfa", "#fb923c", "#34d399", "#f472b6", "#60a5fa"];

function laneStroke(lane: SessionGraphLane): string {
  if (lane.color_key === "main") return "#22d3ee";
  const m = lane.color_key.match(/^fork_(\d+)$/);
  if (m) return FORK_COLORS[Number(m[1]) % FORK_COLORS.length];
  const mm = lane.color_key.match(/^merged_(\d+)$/);
  if (mm) return FORK_COLORS[Number(mm[1]) % FORK_COLORS.length];
  return "#5c5c6e";
}

function isMergedLane(lane: SessionGraphLane): boolean {
  return lane.status === "merged_confirmed" || lane.status === "merged_rejected";
}

function kindLabel(kind: string): string {
  switch (kind) {
    case "checkpoint":
      return "Checkpoint";
    case "branch_fork":
      return "Branch fork";
    case "branch_merge":
      return "Branch merge";
    case "capture_started":
      return "Capture started";
    case "capture_stopped":
      return "Capture stopped";
    case "message_range":
      return "Messages";
    default:
      return kind;
  }
}

function formatTime(at: string): string {
  const d = new Date(at);
  if (Number.isNaN(d.getTime())) return at;
  return d.toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function dotRadius(kind: string): number {
  if (kind === "message_range") return 3;
  if (kind === "checkpoint") return 6;
  if (kind === "capture_started" || kind === "capture_stopped") return 5;
  return 5;
}

interface Props {
  graph: SessionGraph;
  selectedRowId: string | null;
  onSelectRow: (row: SessionGraphRow | null) => void;
  onStartCapture: () => void;
}

export default function SessionGraphView({
  graph,
  selectedRowId,
  onSelectRow,
  onStartCapture,
}: Props) {
  const [hoveredId, setHoveredId] = useState<string | null>(null);

  const laneById = useMemo(
    () => new Map(graph.lanes.map((l) => [l.id, l])),
    [graph.lanes],
  );

  const laneIndex = useMemo(
    () => new Map(graph.lanes.map((l, i) => [l.id, i])),
    [graph.lanes],
  );

  const laneLines = useMemo(() => {
    const lines: Array<{
      laneId: string;
      y1: number;
      y2: number;
      x: number;
      dashed: boolean;
      color: string;
    }> = [];

    for (const lane of graph.lanes) {
      const idx = laneIndex.get(lane.id);
      if (idx === undefined) continue;
      const x = idx * LANE_W + LANE_W / 2;
      const rowIndices = graph.rows
        .map((r, i) => ({ r, i }))
        .filter(({ r }) => r.lane === lane.id);
      for (let j = 1; j < rowIndices.length; j++) {
        const y1 = rowIndices[j - 1].i * ROW_H + ROW_H / 2 + GRAPH_PAD_TOP;
        const y2 = rowIndices[j].i * ROW_H + ROW_H / 2 + GRAPH_PAD_TOP;
        lines.push({
          laneId: lane.id,
          y1,
          y2,
          x,
          dashed: isMergedLane(lane),
          color: laneStroke(lane),
        });
      }
    }
    return lines;
  }, [graph.lanes, graph.rows, laneIndex]);

  const hoveredRow = graph.rows.find((r) => r.id === hoveredId) ?? null;
  const hoveredLane = hoveredRow ? laneById.get(hoveredRow.lane) : undefined;

  if (graph.empty) {
    return (
      <div className="flex h-full min-h-[280px] flex-col items-center justify-center gap-4 px-6 text-center">
        <p className="max-w-md text-sm text-muted-foreground">
          No capture session yet. Start capture to record chat and build a session graph with
          checkpoints and branches.
        </p>
        <button type="button" onClick={onStartCapture} className="cl-btn-accent cl-btn-toolbar">
          <Zap size={13} />
          Start capture
        </button>
      </div>
    );
  }

  const graphWidth = graph.lanes.length * LANE_W + 16;
  const graphHeight = graph.rows.length * ROW_H + GRAPH_PAD_TOP * 2;

  return (
    <div className="relative flex min-h-0 flex-1 overflow-hidden">
      <div className="flex min-h-0 flex-1 overflow-y-auto">
        <div className="sticky left-0 z-10 shrink-0 border-r border-border bg-background/95 px-2 pb-4 pt-3 backdrop-blur-sm">
          <div className="mb-2 font-mono-ui text-[10px] uppercase tracking-widest text-muted-foreground">
            Lanes
          </div>
          <svg
            width={graphWidth}
            height={graphHeight}
            className="block"
            aria-hidden
          >
            {laneLines.map((line, i) => (
              <line
                key={`${line.laneId}-${i}`}
                x1={line.x}
                y1={line.y1}
                x2={line.x}
                y2={line.y2}
                stroke={line.color}
                strokeWidth={2}
                strokeOpacity={line.dashed ? 0.35 : 0.55}
                strokeDasharray={line.dashed ? "4 4" : undefined}
              />
            ))}
            {graph.rows.map((row, rowIndex) => {
              const laneIdx = laneIndex.get(row.lane);
              if (laneIdx === undefined) return null;
              const lane = laneById.get(row.lane);
              if (!lane) return null;
              const cx = laneIdx * LANE_W + LANE_W / 2;
              const cy = rowIndex * ROW_H + ROW_H / 2 + GRAPH_PAD_TOP;
              const r = dotRadius(row.kind);
              const color = laneStroke(lane);
              const merged = isMergedLane(lane);
              const selected = selectedRowId === row.id;
              const hovered = hoveredId === row.id;

              return (
                <g key={row.id}>
                  {row.is_active_head && (
                    <circle
                      cx={cx}
                      cy={cy}
                      r={r + 5}
                      fill="none"
                      stroke="#22d3ee"
                      strokeWidth={1.5}
                      strokeOpacity={0.6}
                    />
                  )}
                  <circle
                    cx={cx}
                    cy={cy}
                    r={selected || hovered ? r + 2 : r}
                    fill={row.kind === "message_range" ? "transparent" : color}
                    stroke={color}
                    strokeWidth={row.kind === "message_range" ? 2 : merged ? 1.5 : 0}
                    strokeOpacity={merged ? 0.45 : 1}
                    strokeDasharray={merged && row.kind !== "message_range" ? "2 2" : undefined}
                    fillOpacity={merged ? 0.5 : 0.9}
                  />
                </g>
              );
            })}
          </svg>
          <div className="mt-2 space-y-1">
            {graph.lanes.map((lane) => (
              <div
                key={lane.id}
                className="flex items-center gap-1.5 font-mono-ui text-[10px] text-muted-foreground"
              >
                <span
                  className="inline-block h-2 w-2 shrink-0 rounded-full"
                  style={{
                    background: laneStroke(lane),
                    opacity: isMergedLane(lane) ? 0.45 : 0.9,
                  }}
                />
                <span className="truncate" title={lane.label}>
                  {lane.label}
                </span>
              </div>
            ))}
          </div>
        </div>

        <div className="min-w-0 flex-1 pb-4 pt-3">
          {graph.rows.map((row) => {
            const selected = selectedRowId === row.id;
            const lane = laneById.get(row.lane);
            return (
              <button
                key={row.id}
                type="button"
                onClick={() => onSelectRow(selected ? null : row)}
                onMouseEnter={() => setHoveredId(row.id)}
                onMouseLeave={() => setHoveredId((id) => (id === row.id ? null : id))}
                className={`flex h-14 w-full items-center gap-3 border-b border-border/60 px-4 text-left transition-colors ${
                  selected
                    ? "bg-[rgba(34,211,238,0.08)]"
                    : "hover:bg-[rgba(255,255,255,0.03)]"
                }`}
              >
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <span className="font-mono-ui text-[10px] uppercase tracking-wide text-muted-foreground">
                      {kindLabel(row.kind)}
                    </span>
                    {lane && (
                      <span
                        className="font-mono-ui rounded-[3px] px-1.5 py-0.5 text-[10px]"
                        style={{
                          background: `${laneStroke(lane)}18`,
                          color: laneStroke(lane),
                        }}
                      >
                        {lane.label}
                      </span>
                    )}
                    {row.message_count > 0 && (
                      <span className="font-mono-ui text-[10px] text-muted-foreground">
                        {row.message_count} msg{row.message_count === 1 ? "" : "s"}
                      </span>
                    )}
                  </div>
                  <p className="mt-0.5 truncate text-sm font-medium text-foreground">
                    {row.primary_label}
                  </p>
                  {row.secondary_label && (
                    <p className="truncate text-xs text-muted-foreground">{row.secondary_label}</p>
                  )}
                </div>
                <span className="shrink-0 font-mono-ui text-[10px] text-muted-foreground/80">
                  {formatTime(row.at)}
                </span>
              </button>
            );
          })}
        </div>
      </div>

      {hoveredRow && (
        <div
          className="pointer-events-none absolute right-4 top-4 z-20 max-w-xs rounded-[3px] border border-border bg-card px-3 py-2 shadow-lg"
          role="tooltip"
        >
          <p className="font-mono-ui text-[10px] uppercase tracking-wide text-muted-foreground">
            {kindLabel(hoveredRow.kind)}
            {hoveredLane ? ` · ${hoveredLane.label}` : ""}
          </p>
          <p className="mt-1 text-xs text-foreground">{formatTime(hoveredRow.at)}</p>
          {hoveredRow.message_count > 0 && (
            <p className="mt-1 text-xs text-muted-foreground">
              {hoveredRow.message_count} message{hoveredRow.message_count === 1 ? "" : "s"}
            </p>
          )}
          {hoveredRow.note && (
            <p className="mt-1 line-clamp-3 text-xs text-muted-foreground">{hoveredRow.note}</p>
          )}
          {(hoveredRow.linked_block_ids?.length ?? 0) > 0 && (
            <p className="mt-1 font-mono-ui text-[10px] text-muted-foreground">
              {hoveredRow.linked_block_ids!.length} linked block
              {hoveredRow.linked_block_ids!.length === 1 ? "" : "s"}
            </p>
          )}
        </div>
      )}
    </div>
  );
}

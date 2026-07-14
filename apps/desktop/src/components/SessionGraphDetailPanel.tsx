import { useEffect, useState } from "react";
import { X } from "lucide-react";
import { fetchSessionLogSlice } from "../api";
import type { CaptureLogMessage, SessionGraphLane, SessionGraphRow } from "../types";

function formatTime(at: string): string {
  const d = new Date(at);
  if (Number.isNaN(d.getTime())) return at;
  return d.toLocaleString();
}

function kindTitle(kind: string): string {
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
      return "Message range";
    default:
      return kind;
  }
}

function roleLabel(role: string): string {
  switch (role) {
    case "user":
      return "User";
    case "assistant":
      return "Assistant";
    case "tool":
      return "Tool";
    case "system":
      return "System";
    default:
      return role;
  }
}

interface Props {
  workspaceId: string;
  row: SessionGraphRow;
  lane?: SessionGraphLane;
  onClose: () => void;
  onOpenBlock?: (blockId: string) => void;
}

export default function SessionGraphDetailPanel({
  workspaceId,
  row,
  lane,
  onClose,
  onOpenBlock,
}: Props) {
  const [messages, setMessages] = useState<CaptureLogMessage[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const showMessages =
    row.kind === "message_range" ||
    row.kind === "checkpoint" ||
    (row.message_count > 0 && row.log_from_seq <= row.log_to_seq);

  useEffect(() => {
    if (!showMessages || row.log_from_seq > row.log_to_seq) {
      setMessages([]);
      return;
    }

    let cancelled = false;
    setLoading(true);
    setError(null);

    fetchSessionLogSlice({
      workspaceId,
      fromSeq: row.log_from_seq,
      toSeq: row.log_to_seq,
      branch: row.lane === "main" ? null : row.lane,
    })
      .then((msgs) => {
        if (!cancelled) setMessages(msgs);
      })
      .catch((e) => {
        if (!cancelled) setError(String(e));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [workspaceId, row, showMessages]);

  return (
    <aside className="flex max-h-full w-80 shrink-0 flex-col overflow-hidden border-l border-border bg-card/50">
      <div className="flex shrink-0 items-start justify-between gap-2 border-b border-border px-4 py-3">
        <div className="min-w-0">
          <h2 className="text-[13px] font-semibold text-foreground">{kindTitle(row.kind)}</h2>
          {lane && (
            <p className="font-mono-ui mt-0.5 text-[10px] text-muted-foreground">{lane.label}</p>
          )}
        </div>
        <button
          type="button"
          onClick={onClose}
          className="shrink-0 rounded-[3px] p-1 text-muted-foreground hover:bg-[rgba(255,255,255,0.06)] hover:text-foreground"
          aria-label="Close detail"
        >
          <X size={14} />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto px-4 py-3">
        <dl className="space-y-3 text-sm">
          <div>
            <dt className="font-mono-ui text-[10px] uppercase tracking-wide text-muted-foreground">
              Time
            </dt>
            <dd className="mt-0.5 text-foreground">{formatTime(row.at)}</dd>
          </div>
          <div>
            <dt className="font-mono-ui text-[10px] uppercase tracking-wide text-muted-foreground">
              Label
            </dt>
            <dd className="mt-0.5 text-foreground">{row.primary_label}</dd>
            {row.secondary_label && (
              <dd className="mt-1 text-xs text-muted-foreground">{row.secondary_label}</dd>
            )}
          </div>
          {row.intent && (
            <div>
              <dt className="font-mono-ui text-[10px] uppercase tracking-wide text-muted-foreground">
                Intent
              </dt>
              <dd className="mt-0.5 text-foreground">{row.intent}</dd>
            </div>
          )}
          {row.note && (
            <div>
              <dt className="font-mono-ui text-[10px] uppercase tracking-wide text-muted-foreground">
                Note
              </dt>
              <dd className="mt-0.5 whitespace-pre-wrap text-foreground">{row.note}</dd>
            </div>
          )}
          {row.git_sha && (
            <div>
              <dt className="font-mono-ui text-[10px] uppercase tracking-wide text-muted-foreground">
                Git SHA
              </dt>
              <dd className="font-mono-ui mt-0.5 text-xs text-foreground">{row.git_sha}</dd>
            </div>
          )}
          {row.merge_outcome && (
            <div>
              <dt className="font-mono-ui text-[10px] uppercase tracking-wide text-muted-foreground">
                Merge outcome
              </dt>
              <dd className="mt-0.5 capitalize text-foreground">{row.merge_outcome}</dd>
            </div>
          )}
          {(row.rejected_paths?.length ?? 0) > 0 && (
            <div>
              <dt className="font-mono-ui text-[10px] uppercase tracking-wide text-muted-foreground">
                Rejected paths
              </dt>
              <dd className="mt-0.5">
                <ul className="list-inside list-disc text-xs text-muted-foreground">
                  {row.rejected_paths!.map((p) => (
                    <li key={p}>{p}</li>
                  ))}
                </ul>
              </dd>
            </div>
          )}
          <div>
            <dt className="font-mono-ui text-[10px] uppercase tracking-wide text-muted-foreground">
              Log seq
            </dt>
            <dd className="font-mono-ui mt-0.5 text-xs text-muted-foreground">
              {row.log_from_seq} – {row.log_to_seq}
              {row.message_count > 0 && ` (${row.message_count} messages)`}
            </dd>
          </div>
        </dl>

        {(row.linked_block_ids?.length ?? 0) > 0 && (
          <div className="mt-4 border-t border-border pt-3">
            <p className="font-mono-ui text-[10px] uppercase tracking-wide text-muted-foreground">
              Linked blocks
            </p>
            <ul className="mt-2 space-y-1">
              {row.linked_block_ids!.map((id) => (
                <li key={id}>
                  {onOpenBlock ? (
                    <button
                      type="button"
                      onClick={() => onOpenBlock(id)}
                      className="font-mono-ui text-xs text-accent hover:underline"
                    >
                      {id.slice(0, 8)}…
                    </button>
                  ) : (
                    <span className="font-mono-ui text-xs text-muted-foreground">
                      {id.slice(0, 8)}…
                    </span>
                  )}
                </li>
              ))}
            </ul>
          </div>
        )}

        {showMessages && (
          <div className="mt-4 border-t border-border pt-3">
            <p className="font-mono-ui text-[10px] uppercase tracking-wide text-muted-foreground">
              Messages
            </p>
            {loading && <p className="mt-2 text-xs text-muted-foreground">Loading…</p>}
            {error && (
              <p className="mt-2 text-xs" style={{ color: "var(--hygiene-warn)" }}>
                {error}
              </p>
            )}
            {!loading && !error && messages.length === 0 && (
              <p className="mt-2 text-xs text-muted-foreground">No messages in this range.</p>
            )}
            <ul className="mt-2 space-y-2">
              {messages.map((m) => (
                <li
                  key={m.id}
                  className="rounded-[3px] border border-border bg-[rgba(255,255,255,0.02)] px-2.5 py-2"
                >
                  <div className="flex items-center justify-between gap-2">
                    <span className="font-mono-ui text-[10px] uppercase text-muted-foreground">
                      {roleLabel(m.role)}
                    </span>
                    <span className="font-mono-ui text-[10px] text-muted-foreground/70">
                      #{m.seq}
                    </span>
                  </div>
                  <p className="mt-1 line-clamp-4 whitespace-pre-wrap text-xs text-foreground">
                    {m.content.trim() || "(empty)"}
                  </p>
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>
    </aside>
  );
}

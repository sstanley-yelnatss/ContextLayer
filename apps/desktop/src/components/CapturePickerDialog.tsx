import type { CaptureCandidate } from "../api";

interface Props {
  open: boolean;
  candidates: CaptureCandidate[];
  rememberScope: boolean;
  onRememberScopeChange: (value: boolean) => void;
  onSelect: (candidate: CaptureCandidate) => void;
  onCancel: () => void;
}

function formatAge(secs: number): string {
  if (secs < 60) return `${secs}s ago`;
  if (secs < 3600) return `${Math.floor(secs / 60)}m ago`;
  if (secs < 86400) return `${Math.floor(secs / 3600)}h ago`;
  return `${Math.floor(secs / 86400)}d ago`;
}

export default function CapturePickerDialog({
  open,
  candidates,
  rememberScope,
  onRememberScopeChange,
  onSelect,
  onCancel,
}: Props) {
  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-[90] flex items-center justify-center bg-black/60 p-4"
      role="presentation"
      onClick={onCancel}
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="capture-picker-title"
        className="w-full max-w-lg rounded-xl border border-zinc-700 bg-zinc-900 p-5 shadow-xl"
        onClick={(e) => e.stopPropagation()}
      >
        <h2 id="capture-picker-title" className="text-base font-medium text-zinc-100">
          Which chat should capture use?
        </h2>
        <p className="mt-2 text-sm leading-relaxed text-zinc-400">
          Chats you used in the last 7 days (up to 30). Pick the thread you are working in.
        </p>
        <ul className="mt-4 max-h-80 space-y-2 overflow-y-auto">
          {candidates.map((c) => (
            <li key={c.transcript_path}>
              <button
                type="button"
                onClick={() => onSelect(c)}
                className="w-full cursor-pointer rounded-lg border border-zinc-700 px-3 py-2.5 text-left text-sm text-zinc-200 transition hover:border-violet-600 hover:bg-violet-950/30"
              >
                <span className="font-medium">{c.label}</span>
                <span className="mt-0.5 block text-xs text-zinc-500">
                  {(c.source === "claude" ? "Claude Code" : "Cursor")} · {c.cursor_project} ·{" "}
                  {formatAge(c.modified_secs_ago)}
                </span>
              </button>
            </li>
          ))}
        </ul>
        <label className="mt-4 flex cursor-pointer items-center gap-2 text-sm text-zinc-400">
          <input
            type="checkbox"
            checked={rememberScope}
            onChange={(e) => onRememberScopeChange(e.target.checked)}
            className="rounded border-zinc-600"
          />
          Remember this chat for this workspace
        </label>
        <div className="mt-5 flex justify-end">
          <button
            type="button"
            onClick={onCancel}
            className="cursor-pointer rounded-lg border border-zinc-700 px-4 py-2 text-sm text-zinc-300 hover:border-zinc-500"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}

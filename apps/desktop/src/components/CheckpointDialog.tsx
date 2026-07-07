import { useEffect, useState } from "react";

export interface CheckpointFormValues {
  intent: string;
  note: string;
  rejectedPaths: string[];
}

interface Props {
  open: boolean;
  onConfirm: (values: CheckpointFormValues) => void;
  onCancel: () => void;
}

export default function CheckpointDialog({ open, onConfirm, onCancel }: Props) {
  const [intent, setIntent] = useState("Ready for PR");
  const [note, setNote] = useState("");
  const [rejectedPaths, setRejectedPaths] = useState("");

  useEffect(() => {
    if (open) {
      setIntent("Ready for PR");
      setNote("");
      setRejectedPaths("");
    }
  }, [open]);

  if (!open) return null;

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    const trimmedIntent = intent.trim();
    if (!trimmedIntent) return;
    onConfirm({
      intent: trimmedIntent,
      note: note.trim(),
      rejectedPaths: rejectedPaths
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean),
    });
  }

  return (
    <div
      className="fixed inset-0 z-[90] flex items-center justify-center bg-black/60 p-4"
      role="presentation"
      onClick={onCancel}
    >
      <form
        role="dialog"
        aria-modal="true"
        aria-labelledby="checkpoint-dialog-title"
        className="w-full max-w-md rounded-xl border border-zinc-700 bg-zinc-900 p-5 shadow-xl"
        onClick={(e) => e.stopPropagation()}
        onSubmit={handleSubmit}
      >
        <h2 id="checkpoint-dialog-title" className="text-base font-medium text-zinc-100">
          Trace checkpoint
        </h2>
        <p className="mt-2 text-sm leading-relaxed text-zinc-400">
          Mark a decision moment. Slices the session log since the last checkpoint for PR trace.
        </p>
        <label className="mt-4 block text-sm text-zinc-400">
          Intent
          <input
            autoFocus
            required
            value={intent}
            onChange={(e) => setIntent(e.target.value)}
            placeholder="e.g. Ready for PR"
            className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-600"
          />
        </label>
        <label className="mt-3 block text-sm text-zinc-400">
          Note <span className="text-zinc-600">(optional)</span>
          <textarea
            value={note}
            onChange={(e) => setNote(e.target.value)}
            rows={2}
            className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-600"
          />
        </label>
        <label className="mt-3 block text-sm text-zinc-400">
          Rejected paths <span className="text-zinc-600">(optional, comma-separated)</span>
          <input
            value={rejectedPaths}
            onChange={(e) => setRejectedPaths(e.target.value)}
            placeholder="e.g. Redis cache, rewrite auth module"
            className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-600"
          />
        </label>
        <div className="mt-5 flex justify-end gap-2">
          <button
            type="button"
            onClick={onCancel}
            className="cursor-pointer rounded-lg border border-zinc-700 px-4 py-2 text-sm text-zinc-300 hover:border-zinc-500"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={!intent.trim()}
            className="cursor-pointer rounded-lg bg-amber-600 px-4 py-2 text-sm font-medium text-white hover:bg-amber-500 disabled:cursor-not-allowed disabled:opacity-50"
          >
            Commit checkpoint
          </button>
        </div>
      </form>
    </div>
  );
}

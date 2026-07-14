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
        className="cl-dialog max-w-md"
        onClick={(e) => e.stopPropagation()}
        onSubmit={handleSubmit}
      >
        <h2 id="checkpoint-dialog-title" className="text-base font-medium text-foreground">
          Trace checkpoint
        </h2>
        <p className="mt-2 text-sm leading-relaxed text-muted-foreground">
          Mark a decision moment. Slices the session log since the last checkpoint for PR trace.
        </p>
        <label className="cl-label mt-4">
          Intent
          <input
            autoFocus
            required
            value={intent}
            onChange={(e) => setIntent(e.target.value)}
            placeholder="e.g. Ready for PR"
            className="cl-input"
          />
        </label>
        <label className="cl-label mt-3">
          Note <span className="text-muted-foreground/70">(optional)</span>
          <textarea value={note} onChange={(e) => setNote(e.target.value)} rows={2} className="cl-input" />
        </label>
        <label className="cl-label mt-3">
          Rejected paths{" "}
          <span className="text-muted-foreground/70">(optional, comma-separated)</span>
          <input
            value={rejectedPaths}
            onChange={(e) => setRejectedPaths(e.target.value)}
            placeholder="e.g. Redis cache, rewrite auth module"
            className="cl-input"
          />
        </label>
        <div className="mt-5 flex justify-end gap-2">
          <button type="button" onClick={onCancel} className="cl-btn-ghost px-4 py-2 text-sm">
            Cancel
          </button>
          <button
            type="submit"
            disabled={!intent.trim()}
            className="cl-btn-accent px-4 py-2 text-sm disabled:cursor-not-allowed disabled:opacity-50"
          >
            Commit checkpoint
          </button>
        </div>
      </form>
    </div>
  );
}

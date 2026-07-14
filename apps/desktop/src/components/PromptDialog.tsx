import { useEffect, useState } from "react";

interface Props {
  open: boolean;
  title: string;
  message?: string;
  label: string;
  placeholder?: string;
  defaultValue?: string;
  confirmLabel: string;
  cancelLabel?: string;
  children?: React.ReactNode;
  onConfirm: (value: string) => void;
  onCancel: () => void;
}

export default function PromptDialog({
  open,
  title,
  message,
  label,
  placeholder,
  defaultValue = "",
  confirmLabel,
  cancelLabel = "Cancel",
  children,
  onConfirm,
  onCancel,
}: Props) {
  const [value, setValue] = useState(defaultValue);

  useEffect(() => {
    if (open) setValue(defaultValue);
  }, [open, defaultValue]);

  if (!open) return null;

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    onConfirm(value.trim());
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
        aria-labelledby="prompt-dialog-title"
        className="cl-dialog max-w-md"
        onClick={(e) => e.stopPropagation()}
        onSubmit={handleSubmit}
      >
        <h2 id="prompt-dialog-title" className="text-base font-medium text-foreground">
          {title}
        </h2>
        {message && (
          <p className="mt-2 text-sm leading-relaxed text-muted-foreground">{message}</p>
        )}
        {children}
        <label className="cl-label mt-4">
          {label}
          <input
            autoFocus
            value={value}
            onChange={(e) => setValue(e.target.value)}
            placeholder={placeholder}
            className="cl-input"
          />
        </label>
        <div className="mt-5 flex justify-end gap-2">
          <button
            type="button"
            onClick={onCancel}
            className="cl-btn-ghost px-4 py-2 text-sm"
          >
            {cancelLabel}
          </button>
          <button
            type="submit"
            className="cl-btn-export px-4 py-2 text-sm"
          >
            {confirmLabel}
          </button>
        </div>
      </form>
    </div>
  );
}

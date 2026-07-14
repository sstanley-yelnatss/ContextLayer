import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from "react";

export type ToastKind = "success" | "error" | "info";

export type ToastAction = {
  label: string;
  onClick: () => void;
};

export type ToastInput = {
  message: string;
  kind?: ToastKind;
  durationMs?: number;
  action?: ToastAction;
};

type ToastItem = ToastInput & {
  id: string;
  kind: ToastKind;
  durationMs: number;
};

type ToastContextValue = {
  showToast: (input: string | ToastInput) => void;
};

const ToastContext = createContext<ToastContextValue | null>(null);

const DEFAULT_DURATION_MS = 6000;

const KIND_STYLES: Record<ToastKind, string> = {
  success:
    "border-[rgba(52,211,153,0.25)] bg-card text-foreground shadow-lg shadow-black/40",
  error:
    "border-[rgba(248,113,113,0.25)] bg-card text-foreground shadow-lg shadow-black/40",
  info: "border-border bg-card text-foreground shadow-lg shadow-black/40",
};

function normalizeToast(input: string | ToastInput): Omit<ToastItem, "id"> {
  if (typeof input === "string") {
    return { message: input, kind: "success", durationMs: DEFAULT_DURATION_MS };
  }
  return {
    message: input.message,
    kind: input.kind ?? "success",
    durationMs: input.durationMs ?? DEFAULT_DURATION_MS,
    action: input.action,
  };
}

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<ToastItem[]>([]);

  const dismiss = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  const showToast = useCallback(
    (input: string | ToastInput) => {
      const normalized = normalizeToast(input);
      const id = crypto.randomUUID();
      setToasts((prev) => [...prev, { ...normalized, id }]);
      window.setTimeout(() => dismiss(id), normalized.durationMs);
    },
    [dismiss],
  );

  const value = useMemo(() => ({ showToast }), [showToast]);

  return (
    <ToastContext.Provider value={value}>
      {children}
      <div
        className="pointer-events-none fixed bottom-4 right-4 z-[100] flex w-[min(100vw-2rem,22rem)] flex-col gap-2"
        aria-live="polite"
        aria-relevant="additions"
      >
        {toasts.map((toast) => (
          <div
            key={toast.id}
            role="status"
            className={`pointer-events-auto rounded-[4px] border px-4 py-3 text-sm leading-snug ${KIND_STYLES[toast.kind]}`}
          >
            <div className="flex items-start justify-between gap-3">
              <p className="min-w-0 flex-1">{toast.message}</p>
              <button
                type="button"
                onClick={() => dismiss(toast.id)}
                className="shrink-0 cursor-pointer text-xs opacity-60 hover:opacity-100"
                aria-label="Dismiss"
              >
                ✕
              </button>
            </div>
            {toast.action && (
              <button
                type="button"
                onClick={() => {
                  toast.action?.onClick();
                  dismiss(toast.id);
                }}
                className="mt-2 cursor-pointer text-xs font-medium underline underline-offset-2 opacity-90 hover:opacity-100"
              >
                {toast.action.label}
              </button>
            )}
          </div>
        ))}
      </div>
    </ToastContext.Provider>
  );
}

export function useToast() {
  const ctx = useContext(ToastContext);
  if (!ctx) {
    throw new Error("useToast must be used within ToastProvider");
  }
  return ctx;
}

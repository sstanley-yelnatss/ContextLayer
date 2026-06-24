import { useEffect } from "react";

/** Clear a message after `ms` (status toasts, etc.). */
export function useAutoDismiss(
  message: string,
  onClear: () => void,
  ms = 3500,
) {
  useEffect(() => {
    if (!message) return;
    const timer = window.setTimeout(onClear, ms);
    return () => window.clearTimeout(timer);
  }, [message, onClear, ms]);
}

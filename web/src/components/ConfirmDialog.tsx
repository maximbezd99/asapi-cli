import { useEffect, useId, useRef } from "react";
import { AlertTriangle } from "lucide-react";
import { createPortal } from "react-dom";

interface Props {
  title: string;
  description: string;
  confirmLabel: string;
  busy?: boolean;
  alternateLabel?: string;
  onAlternateConfirm?: () => void;
  onCancel: () => void;
  onConfirm: () => void;
}

export default function ConfirmDialog({
  title,
  description,
  confirmLabel,
  busy = false,
  alternateLabel,
  onAlternateConfirm,
  onCancel,
  onConfirm,
}: Props) {
  const titleId = useId();
  const descriptionId = useId();
  const dialogRef = useRef<HTMLDivElement | null>(null);
  const cancelRef = useRef<HTMLButtonElement | null>(null);

  useEffect(() => {
    const previousFocus = document.activeElement as HTMLElement | null;
    cancelRef.current?.focus();
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape" && !busy) {
        event.preventDefault();
        onCancel();
      }
      if (event.key !== "Tab") return;
      const controls = Array.from(
        dialogRef.current?.querySelectorAll<HTMLElement>(
          'button:not([disabled]), [href], input:not([disabled])',
        ) ?? [],
      );
      if (!controls.length) return;
      const first = controls[0];
      const last = controls[controls.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    };
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
      previousFocus?.focus();
    };
  }, [busy, onCancel]);

  return createPortal(
    <div
      className="dialog-backdrop"
      onPointerDown={(event) => {
        if (event.currentTarget === event.target && !busy) onCancel();
      }}
    >
      <div
        ref={dialogRef}
        className="confirm-dialog"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby={titleId}
        aria-describedby={descriptionId}
      >
        <div className="confirm-dialog-mark" aria-hidden="true">
          <AlertTriangle size={20} />
        </div>
        <div>
          <h2 id={titleId}>{title}</h2>
          <p id={descriptionId}>{description}</p>
        </div>
        <div className="confirm-dialog-actions">
          {alternateLabel && onAlternateConfirm ? (
            <button
              className="destructive-all"
              type="button"
              disabled={busy}
              onClick={onAlternateConfirm}
            >
              {busy ? "Deleting…" : alternateLabel}
            </button>
          ) : null}
          <button
            ref={cancelRef}
            type="button"
            disabled={busy}
            onClick={onCancel}
          >
            Cancel
          </button>
          <button
            className="destructive"
            type="button"
            disabled={busy}
            onClick={onConfirm}
          >
            {busy ? "Deleting…" : confirmLabel}
          </button>
        </div>
      </div>
    </div>,
    document.body,
  );
}

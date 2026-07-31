import { Check, Copy, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";

type CopyStatus = "idle" | "copied" | "error";

async function copyText(value: string) {
  if (navigator.clipboard && window.isSecureContext) {
    try {
      await navigator.clipboard.writeText(value);
      return;
    } catch {
      // Some browsers expose the API but reject it without a user permission.
    }
  }

  const previousFocus = document.activeElement as HTMLElement | null;
  const field = document.createElement("textarea");
  field.value = value;
  field.style.position = "fixed";
  field.style.opacity = "0";
  document.body.appendChild(field);
  try {
    field.select();
    if (!document.execCommand("copy")) {
      throw new Error("clipboard copy failed");
    }
  } finally {
    field.remove();
    previousFocus?.focus();
  }
}

export default function AppIdCopyButton({ appId }: { appId: number }) {
  const [status, setStatus] = useState<CopyStatus>("idle");
  const resetTimer = useRef<number | null>(null);

  useEffect(
    () => () => {
      if (resetTimer.current != null) window.clearTimeout(resetTimer.current);
    },
    [],
  );

  const copy = async () => {
    if (resetTimer.current != null) window.clearTimeout(resetTimer.current);
    try {
      await copyText(String(appId));
      setStatus("copied");
    } catch {
      setStatus("error");
    }
    resetTimer.current = window.setTimeout(() => setStatus("idle"), 1800);
  };

  const message =
    status === "copied"
      ? `Copied App Store ID ${appId}`
      : status === "error"
        ? `Could not copy App Store ID ${appId}. Try again.`
        : "";

  return (
    <button
      type="button"
      className={`ranking-id-copy ${status}`}
      onClick={() => void copy()}
      aria-label={
        status === "error"
          ? `Copy failed. Retry copying App Store ID ${appId}`
          : status === "copied"
            ? `Copied App Store ID ${appId}`
            : `Copy App Store ID ${appId}`
      }
      title={
        status === "error" ? "Copy failed — try again" : "Copy App Store ID"
      }
    >
      {status === "copied" ? (
        <Check size={10} />
      ) : status === "error" ? (
        <X size={10} />
      ) : (
        <Copy size={10} />
      )}
      {status === "error" ? "Retry " : ""}ID {appId}
      <span className="visually-hidden" role="status" aria-live="polite">
        {message}
      </span>
    </button>
  );
}

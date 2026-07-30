import { useCallback, useEffect, useId, useRef, useState } from "react";
import { ChevronLeft, ChevronRight, X } from "lucide-react";
import { createPortal } from "react-dom";

interface Props {
  images: string[];
  initialIndex: number;
  appName: string;
  onClose: () => void;
}

export default function ScreenshotViewer({
  images,
  initialIndex,
  appName,
  onClose,
}: Props) {
  const [index, setIndex] = useState(initialIndex);
  const titleId = useId();
  const dialogRef = useRef<HTMLDivElement | null>(null);
  const closeRef = useRef<HTMLButtonElement | null>(null);

  const select = useCallback(
    (next: number) => {
      setIndex((next + images.length) % images.length);
    },
    [images.length],
  );

  useEffect(() => {
    const previousFocus = document.activeElement as HTMLElement | null;
    closeRef.current?.focus();
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        onClose();
      } else if (event.key === "ArrowLeft") {
        event.preventDefault();
        setIndex((current) => (current - 1 + images.length) % images.length);
      } else if (event.key === "ArrowRight") {
        event.preventDefault();
        setIndex((current) => (current + 1) % images.length);
      } else if (event.key === "Tab") {
        const controls = Array.from(
          dialogRef.current?.querySelectorAll<HTMLElement>(
            'button:not([disabled]), [href], [tabindex]:not([tabindex="-1"])',
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
      }
    };
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
      previousFocus?.focus();
    };
  }, [images.length, onClose]);

  return createPortal(
    <div className="screenshot-viewer-backdrop">
      <div
        ref={dialogRef}
        className="screenshot-viewer"
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
      >
        <header>
          <h2 id={titleId}>{appName} screenshots</h2>
          <span>
            {index + 1} / {images.length}
          </span>
          <button
            ref={closeRef}
            type="button"
            aria-label="Close screenshot viewer"
            onClick={onClose}
          >
            <X size={16} />
          </button>
        </header>
        <div className="screenshot-viewer-stage">
          <button
            type="button"
            className="screenshot-viewer-arrow previous"
            aria-label="Previous screenshot"
            onClick={() => select(index - 1)}
          >
            <ChevronLeft size={21} />
          </button>
          <img
            src={images[index]}
            alt={`${appName} screenshot ${index + 1} of ${images.length}`}
          />
          <button
            type="button"
            className="screenshot-viewer-arrow next"
            aria-label="Next screenshot"
            onClick={() => select(index + 1)}
          >
            <ChevronRight size={21} />
          </button>
        </div>
        <div className="screenshot-viewer-strip" aria-label="Choose screenshot">
          {images.map((image, imageIndex) => (
            <button
              type="button"
              className={imageIndex === index ? "active" : ""}
              aria-label={`Show screenshot ${imageIndex + 1}`}
              aria-current={imageIndex === index ? "true" : undefined}
              onClick={() => select(imageIndex)}
              key={image}
            >
              <img src={image} alt="" loading="lazy" />
              <span>{String(imageIndex + 1).padStart(2, "0")}</span>
            </button>
          ))}
        </div>
      </div>
    </div>,
    document.body,
  );
}

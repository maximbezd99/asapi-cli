import {
  useEffect,
  useId,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import type { CSSProperties, KeyboardEvent, ReactNode } from "react";
import { Check, ChevronDown, Search } from "lucide-react";
import { createPortal } from "react-dom";

export interface PickerOption {
  value: string;
  label: string;
  triggerLabel?: string;
  meta?: string;
  icon?: ReactNode;
}

interface Props {
  value: string | null;
  options: PickerOption[];
  onChange: (value: string) => void;
  ariaLabel: string;
  className?: string;
  disabled?: boolean;
  placeholder?: string;
  triggerContent?: ReactNode;
  iconOnly?: boolean;
  showChevron?: boolean;
  searchPlaceholder?: string;
  portalContainer?: HTMLElement | null;
}

interface MenuPosition {
  top: number;
  left: number;
  width: number;
  maxHeight: number;
  above: boolean;
}

export default function Picker({
  value,
  options,
  onChange,
  ariaLabel,
  className = "",
  disabled = false,
  placeholder = "Select",
  triggerContent,
  iconOnly = false,
  showChevron = !iconOnly,
  searchPlaceholder = "Search",
  portalContainer,
}: Props) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);
  const [position, setPosition] = useState<MenuPosition | null>(null);
  const triggerRef = useRef<HTMLButtonElement | null>(null);
  const menuRef = useRef<HTMLDivElement | null>(null);
  const searchRef = useRef<HTMLInputElement | null>(null);
  const listboxId = useId();
  const searchable = options.length > 5;
  const selected = options.find((option) => option.value === value);

  const filteredOptions = useMemo(() => {
    const needle = query.trim().toLocaleLowerCase();
    if (!needle) return options;
    return options.filter((option) =>
      `${option.label} ${option.meta ?? ""} ${option.value}`
        .toLocaleLowerCase()
        .includes(needle),
    );
  }, [options, query]);

  const updatePosition = () => {
    const trigger = triggerRef.current;
    if (!trigger) return;
    const rect = trigger.getBoundingClientRect();
    const menuWidth = Math.min(
      Math.max(rect.width, iconOnly ? 238 : 210),
      window.innerWidth - 16,
    );
    const menuHeight = Math.min(
      menuRef.current?.getBoundingClientRect().height ?? 300,
      330,
    );
    const below = window.innerHeight - rect.bottom - 8;
    const aboveSpace = rect.top - 8;
    const above = below < Math.min(menuHeight, 190) && aboveSpace > below;
    const maxHeight = Math.max(116, Math.min(330, above ? aboveSpace : below));
    const preferredLeft = iconOnly ? rect.right - menuWidth : rect.left;
    const left = Math.max(
      8,
      Math.min(preferredLeft, window.innerWidth - menuWidth - 8),
    );
    setPosition({
      top: above
        ? Math.max(8, rect.top - Math.min(menuHeight, maxHeight) - 4)
        : rect.bottom + 4,
      left,
      width: menuWidth,
      maxHeight,
      above,
    });
  };

  const close = (restoreFocus = false) => {
    setOpen(false);
    setQuery("");
    setPosition(null);
    if (restoreFocus) requestAnimationFrame(() => triggerRef.current?.focus());
  };

  const choose = (option: PickerOption) => {
    onChange(option.value);
    close(true);
  };

  const openPicker = (initialQuery = "") => {
    if (disabled || !options.length) return;
    setQuery(initialQuery);
    const selectedIndex = options.findIndex((option) => option.value === value);
    setActiveIndex(Math.max(0, selectedIndex));
    setOpen(true);
  };

  useLayoutEffect(() => {
    if (!open) return;
    updatePosition();
    const frame = requestAnimationFrame(updatePosition);
    return () => cancelAnimationFrame(frame);
  }, [open, filteredOptions.length]);

  useEffect(() => {
    if (!open) return;
    const handlePointerDown = (event: PointerEvent) => {
      const target = event.target as Node;
      if (
        triggerRef.current?.contains(target) ||
        menuRef.current?.contains(target)
      ) {
        return;
      }
      close();
    };
    const handleWindowChange = () => updatePosition();
    const handleKeyDown = (event: globalThis.KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        close(true);
      } else if (event.key === "Tab") {
        close();
      }
    };
    document.addEventListener("pointerdown", handlePointerDown, true);
    document.addEventListener("keydown", handleKeyDown);
    window.addEventListener("resize", handleWindowChange);
    window.addEventListener("scroll", handleWindowChange, true);
    return () => {
      document.removeEventListener("pointerdown", handlePointerDown, true);
      document.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("resize", handleWindowChange);
      window.removeEventListener("scroll", handleWindowChange, true);
    };
  }, [open]);

  useEffect(() => {
    if (!open || !searchable) return;
    const frame = requestAnimationFrame(() => {
      const input = searchRef.current;
      input?.focus();
      if (!input) return;
      if (input.value) {
        input.setSelectionRange(input.value.length, input.value.length);
      } else {
        input.select();
      }
    });
    return () => cancelAnimationFrame(frame);
  }, [open, searchable]);

  useEffect(() => {
    setActiveIndex((index) =>
      Math.max(0, Math.min(index, filteredOptions.length - 1)),
    );
  }, [filteredOptions.length]);

  useEffect(() => {
    if (!open) return;
    menuRef.current
      ?.querySelector<HTMLElement>(".picker-options > button.active")
      ?.scrollIntoView({ block: "nearest" });
  }, [activeIndex, filteredOptions.length, open]);

  const handleTriggerKeyDown = (event: KeyboardEvent<HTMLButtonElement>) => {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      if (!open) {
        openPicker();
      } else if (!searchable) {
        setActiveIndex((index) => {
          if (!filteredOptions.length) return 0;
          return event.key === "ArrowDown"
            ? (index + 1) % filteredOptions.length
            : (index - 1 + filteredOptions.length) % filteredOptions.length;
        });
      }
      return;
    }
    if (
      open &&
      !searchable &&
      (event.key === "Enter" || event.key === " ")
    ) {
      event.preventDefault();
      const option = filteredOptions[activeIndex];
      if (option) choose(option);
      return;
    }
    if (open && !searchable && event.key === "Home") {
      event.preventDefault();
      setActiveIndex(0);
      return;
    }
    if (open && !searchable && event.key === "End") {
      event.preventDefault();
      setActiveIndex(Math.max(0, filteredOptions.length - 1));
      return;
    }
    if (
      searchable &&
      !open &&
      event.key.length === 1 &&
      !event.ctrlKey &&
      !event.metaKey &&
      !event.altKey
    ) {
      event.preventDefault();
      openPicker(event.key);
    }
  };

  const handleMenuKeyDown = (event: KeyboardEvent) => {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      setActiveIndex((index) =>
        filteredOptions.length ? (index + 1) % filteredOptions.length : 0,
      );
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      setActiveIndex((index) =>
        filteredOptions.length
          ? (index - 1 + filteredOptions.length) % filteredOptions.length
          : 0,
      );
    } else if (event.key === "Home") {
      event.preventDefault();
      setActiveIndex(0);
    } else if (event.key === "End") {
      event.preventDefault();
      setActiveIndex(Math.max(0, filteredOptions.length - 1));
    } else if (event.key === "Enter") {
      event.preventDefault();
      const option = filteredOptions[activeIndex];
      if (option) choose(option);
    }
  };

  const menuStyle = position
    ? ({
        top: position.top,
        left: position.left,
        width: position.width,
        "--picker-max-height": `${position.maxHeight}px`,
      } as CSSProperties)
    : undefined;

  return (
    <span
      className={`picker ${iconOnly ? "picker-icon-only" : ""} ${className}`}
    >
      <button
        ref={triggerRef}
        type="button"
        className="picker-trigger"
        aria-label={ariaLabel}
        aria-haspopup="listbox"
        aria-expanded={open}
        aria-controls={open ? listboxId : undefined}
        disabled={disabled || !options.length}
        onClick={() => (open ? close() : openPicker())}
        onKeyDown={handleTriggerKeyDown}
      >
        {triggerContent ?? (
          <>
            {selected?.icon ? (
              <span className="picker-value-icon">{selected.icon}</span>
            ) : null}
            <span className="picker-value">
              {selected?.triggerLabel ?? selected?.label ?? placeholder}
            </span>
          </>
        )}
        {showChevron ? (
          <ChevronDown
            className={open ? "picker-chevron open" : "picker-chevron"}
            size={13}
          />
        ) : null}
      </button>

      {open
        ? createPortal(
            <div
              ref={menuRef}
              className={`picker-menu ${position?.above ? "above" : ""}`}
              style={menuStyle}
              onKeyDown={handleMenuKeyDown}
            >
              {searchable ? (
                <label className="picker-search">
                  <Search size={13} />
                  <input
                    ref={searchRef}
                    value={query}
                    onChange={(event) => {
                      setQuery(event.target.value);
                      setActiveIndex(0);
                    }}
                    placeholder={searchPlaceholder}
                    aria-label={`Search ${ariaLabel.toLocaleLowerCase()}`}
                  />
                  <kbd>ESC</kbd>
                </label>
              ) : null}
              <div
                id={listboxId}
                className="picker-options"
                role="listbox"
                aria-label={ariaLabel}
              >
                {filteredOptions.map((option, index) => (
                  <button
                    type="button"
                    role="option"
                    aria-selected={option.value === value}
                    className={[
                      option.value === value ? "selected" : "",
                      index === activeIndex ? "active" : "",
                    ]
                      .filter(Boolean)
                      .join(" ")}
                    key={option.value}
                    onPointerMove={() => setActiveIndex(index)}
                    onClick={() => choose(option)}
                  >
                    {option.icon ? (
                      <span className="picker-option-icon">{option.icon}</span>
                    ) : null}
                    <span className="picker-option-copy">
                      <strong>{option.label}</strong>
                      {option.meta ? <small>{option.meta}</small> : null}
                    </span>
                    {option.value === value ? (
                      <Check className="picker-check" size={13} />
                    ) : null}
                  </button>
                ))}
                {!filteredOptions.length ? (
                  <div className="picker-empty">No matching options</div>
                ) : null}
              </div>
            </div>,
            portalContainer ?? document.body,
          )
        : null}
    </span>
  );
}

import { useEffect, useId, useMemo, useRef, useState } from "react";
import { ArrowUpRight, RefreshCw, Search, Star, X } from "lucide-react";
import { createPortal } from "react-dom";
import { formatCount, formatDate } from "../format";
import type { RankedApp } from "../types";

interface Props {
  apps: RankedApp[];
  country: string;
  keyword: string;
  onRefresh: () => Promise<void>;
  onClose: () => void;
}

export default function RankingResultsDialog({
  apps,
  country,
  keyword,
  onRefresh,
  onClose,
}: Props) {
  const [query, setQuery] = useState("");
  const [refreshingDetails, setRefreshingDetails] = useState(false);
  const [refreshError, setRefreshError] = useState("");
  const titleId = useId();
  const dialogRef = useRef<HTMLDivElement | null>(null);
  const searchRef = useRef<HTMLInputElement | null>(null);
  const attemptedRefresh = useRef(false);
  const hasLegacyRows =
    apps.length > 0 &&
    apps.every(
      (app) =>
        app.released_at == null &&
        app.version_released_at == null &&
        app.rating == null &&
        app.rating_count == null,
    );

  const filtered = useMemo(() => {
    const normalized = query.trim().toLocaleLowerCase();
    if (!normalized) return apps;
    return apps.filter((app) =>
      [app.name, app.developer_name, String(app.apple_id)]
        .filter(Boolean)
        .some((value) =>
          String(value).toLocaleLowerCase().includes(normalized),
        ),
    );
  }, [apps, query]);

  const refreshLegacyRows = async () => {
    setRefreshingDetails(true);
    setRefreshError("");
    try {
      await onRefresh();
    } catch (reason) {
      setRefreshError((reason as Error).message);
    } finally {
      setRefreshingDetails(false);
    }
  };

  useEffect(() => {
    if (!hasLegacyRows || attemptedRefresh.current) return;
    attemptedRefresh.current = true;
    void refreshLegacyRows();
  }, [hasLegacyRows]);

  useEffect(() => {
    const previousFocus = document.activeElement as HTMLElement | null;
    searchRef.current?.focus();
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        onClose();
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
  }, [onClose]);

  return createPortal(
    <div className="ranking-results-backdrop">
      <div
        ref={dialogRef}
        className="ranking-results-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
      >
        <header>
          <div>
            <h2 id={titleId}>Apps ranking for “{keyword}”</h2>
            {refreshError ? (
              <span className="ranking-refresh-error" role="alert">
                Details unavailable
                <button
                  type="button"
                  disabled={refreshingDetails}
                  onClick={() => void refreshLegacyRows()}
                >
                  Retry
                </button>
              </span>
            ) : (
              <span role={refreshingDetails ? "status" : undefined}>
                {refreshingDetails ? (
                  <>
                    <RefreshCw className="spinning" size={10} />
                    Updating cached app details
                  </>
                ) : (
                  <>
                    {country.toUpperCase()} / {apps.length} cached results
                  </>
                )}
              </span>
            )}
          </div>
          <label>
            <Search size={14} />
            <input
              ref={searchRef}
              type="search"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="Search name, developer, or ID"
            />
          </label>
          <button
            type="button"
            aria-label="Close ranking results"
            onClick={onClose}
          >
            <X size={16} />
          </button>
        </header>
        <div className="ranking-results-table-wrap">
          <table className="ranking-results-table">
            <thead>
              <tr>
                <th>#</th>
                <th>App</th>
                <th>First release</th>
                <th>Last release</th>
                <th>Rating</th>
                <th>Ratings</th>
                <th aria-label="App Store link" />
              </tr>
            </thead>
            <tbody>
              {filtered.map((app) => (
                <tr key={app.apple_id}>
                  <td>{app.position}</td>
                  <td>
                    <span className="ranking-result-app">
                      {app.icon_url ? (
                        <img src={app.icon_url} alt="" loading="lazy" />
                      ) : (
                        <i>{app.name.slice(0, 1)}</i>
                      )}
                      <span>
                        <strong>{app.name}</strong>
                        <small>{app.developer_name ?? "Unknown developer"}</small>
                      </span>
                    </span>
                  </td>
                  <td>
                    {app.released_at
                      ? formatDate(app.released_at)
                      : refreshingDetails
                        ? "Loading…"
                        : "—"}
                  </td>
                  <td>
                    {app.version_released_at
                      ? formatDate(app.version_released_at)
                      : refreshingDetails
                        ? "Loading…"
                        : "—"}
                  </td>
                  <td>
                    <span className="ranking-result-rating">
                      <Star size={11} fill="currentColor" />
                      {app.rating?.toFixed(1) ?? "—"}
                    </span>
                  </td>
                  <td>{formatCount(app.rating_count)}</td>
                  <td>
                    <a
                      href={`https://apps.apple.com/${country}/app/id${app.apple_id}`}
                      target="_blank"
                      rel="noreferrer"
                      aria-label={`Open ${app.name} in the App Store`}
                    >
                      <ArrowUpRight size={14} />
                    </a>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          {!filtered.length ? (
            <div className="ranking-results-empty">
              No cached apps match “{query}”.
            </div>
          ) : null}
        </div>
      </div>
    </div>,
    document.body,
  );
}

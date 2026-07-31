import {
  useCallback,
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
} from "react";
import { ArrowUpRight, Search, Star, X } from "lucide-react";
import { createPortal } from "react-dom";
import { api } from "../api";
import {
  countryFlag,
  countryLabel,
  formatCount,
  formatDate,
} from "../format";
import type { Country, SearchApp } from "../types";
import { useAppEstimates } from "../useAppEstimates";
import AppIdCopyButton from "./AppIdCopyButton";
import Picker from "./Picker";

interface Props {
  countries: Country[];
  onClose: () => void;
}

export default function AppSearchDialog({ countries, onClose }: Props) {
  const initialCountry = localStorage.getItem("asapi-search-country") ?? "us";
  const [country, setCountry] = useState(
    countries.some((item) => item.code === initialCountry)
      ? initialCountry
      : (countries[0]?.code ?? "us"),
  );
  const [query, setQuery] = useState("");
  const [submittedQuery, setSubmittedQuery] = useState("");
  const [results, setResults] = useState<SearchApp[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const titleId = useId();
  const dialogRef = useRef<HTMLDivElement | null>(null);
  const [dialogElement, setDialogElement] = useState<HTMLDivElement | null>(
    null,
  );
  const inputRef = useRef<HTMLInputElement | null>(null);
  const requestVersion = useRef(0);
  const appIds = useMemo(
    () => results.map((result) => result.app_id),
    [results],
  );
  const {
    estimates,
    loading: estimatesLoading,
    error: estimatesError,
    retry: retryEstimates,
  } = useAppEstimates(appIds);
  const captureDialog = useCallback((node: HTMLDivElement | null) => {
    dialogRef.current = node;
    setDialogElement(node);
  }, []);

  const search = async (term: string, storefront: string) => {
    const normalized = term.trim();
    if (!normalized) return;
    const version = ++requestVersion.current;
    setLoading(true);
    setError("");
    setResults([]);
    try {
      const items = await api.searchApps(normalized, storefront, 200);
      if (requestVersion.current !== version) return;
      setResults(items);
      setSubmittedQuery(normalized);
    } catch (reason) {
      if (requestVersion.current !== version) return;
      setError((reason as Error).message);
    } finally {
      if (requestVersion.current === version) setLoading(false);
    }
  };

  const changeCountry = (nextCountry: string) => {
    setCountry(nextCountry);
    localStorage.setItem("asapi-search-country", nextCountry);
    if (submittedQuery) void search(submittedQuery, nextCountry);
  };

  useEffect(() => {
    const previousFocus = document.activeElement as HTMLElement | null;
    inputRef.current?.focus();
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape" && document.querySelector(".picker-menu")) {
        return;
      }
      if (event.key === "Escape" && !event.defaultPrevented) {
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
      requestVersion.current += 1;
      document.removeEventListener("keydown", handleKeyDown);
      previousFocus?.focus();
    };
  }, [onClose]);

  return createPortal(
    <div className="ranking-results-backdrop">
      <div
        ref={captureDialog}
        className="ranking-results-dialog app-search-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
      >
        <header>
          <div>
            <h2 id={titleId}>Search App Store</h2>
            <span role={loading || estimatesLoading ? "status" : undefined}>
              {loading
                ? `Searching ${country.toUpperCase()}`
                : submittedQuery
                  ? `${country.toUpperCase()} / ${results.length} results`
                  : "Choose a storefront and enter a search"}
            </span>
            {estimatesLoading ? (
              <span role="status">Fetching worldwide estimates</span>
            ) : estimatesError ? (
              <span className="ranking-refresh-error" role="alert">
                Estimates unavailable
                <button type="button" onClick={retryEstimates}>
                  Retry
                </button>
              </span>
            ) : results.length ? (
              <span>Worldwide / last month</span>
            ) : null}
          </div>
          <form
            className="app-search-form"
            onSubmit={(event) => {
              event.preventDefault();
              void search(query, country);
            }}
          >
            <label>
              <Search size={14} />
              <input
                ref={inputRef}
                type="search"
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                placeholder="App name or developer"
                aria-label="Search App Store"
              />
            </label>
            <Picker
              value={country}
              options={countries.map((item) => ({
                value: item.code,
                label: item.name,
                triggerLabel: item.code.toUpperCase(),
                meta: item.code.toUpperCase(),
                icon: countryFlag(item.code),
              }))}
              onChange={changeCountry}
              ariaLabel="Search storefront"
              className="app-search-country-picker"
              searchPlaceholder="Search storefronts"
              disabled={loading}
              portalContainer={dialogElement}
            />
            <button type="submit" disabled={loading || !query.trim()}>
              Search
            </button>
          </form>
          <button
            type="button"
            aria-label="Close App Store search"
            onClick={onClose}
          >
            <X size={16} />
          </button>
        </header>

        <div className="ranking-results-table-wrap">
          {error ? (
            <div className="app-search-error" role="alert">
              <strong>Search unavailable</strong>
              <span>{error}</span>
              <button
                type="button"
                disabled={loading}
                onClick={() => void search(query, country)}
              >
                Retry
              </button>
            </div>
          ) : results.length ? (
            <table className="ranking-results-table">
              <thead>
                <tr>
                  <th className="result-position">#</th>
                  <th className="result-app">App</th>
                  <th className="result-date">First release</th>
                  <th className="result-date">Last release</th>
                  <th className="result-rating">Rating</th>
                  <th className="result-ratings">Ratings</th>
                  <th className="result-estimate">Downloads</th>
                  <th className="result-estimate">Revenue</th>
                  <th className="result-link" aria-label="App Store link" />
                </tr>
              </thead>
              <tbody>
                {results.map((app) => (
                  <tr key={app.app_id}>
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
                          <span className="ranking-result-app-meta">
                            <small>
                              {app.developer_name ?? "Unknown developer"}
                            </small>
                            <AppIdCopyButton appId={app.app_id} />
                          </span>
                        </span>
                      </span>
                    </td>
                    <td>
                      {app.released_at ? formatDate(app.released_at) : "—"}
                    </td>
                    <td>
                      {app.version_released_at
                        ? formatDate(app.version_released_at)
                        : "—"}
                    </td>
                    <td>
                      <span className="ranking-result-rating">
                        <Star size={11} fill="currentColor" />
                        {app.rating?.toFixed(1) ?? "—"}
                      </span>
                    </td>
                    <td>{formatCount(app.rating_count)}</td>
                    <td title="Estimated worldwide downloads last month">
                      {estimates.get(app.app_id)
                        ?.humanized_worldwide_last_month_downloads?.string ??
                        (estimatesLoading ? "Loading…" : "—")}
                    </td>
                    <td title="Estimated worldwide revenue last month">
                      {estimates.get(app.app_id)
                        ?.humanized_worldwide_last_month_revenue?.string ??
                        (estimatesLoading ? "Loading…" : "—")}
                    </td>
                    <td>
                      <a
                        href={
                          app.app_store_url ??
                          `https://apps.apple.com/${country}/app/id${app.app_id}`
                        }
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
          ) : loading ? (
            <div className="app-search-empty" role="status">
              <span className="registry-loader" aria-hidden="true">
                <i />
                <i />
                <i />
              </span>
              <strong>Searching {countryLabel(country)}</strong>
              <p>Reading the current storefront result set.</p>
            </div>
          ) : submittedQuery ? (
            <div className="app-search-empty">
              <Search size={22} />
              <strong>No apps found for “{submittedQuery}”</strong>
              <p>Try a broader name or another storefront.</p>
            </div>
          ) : (
            <div className="app-search-empty">
              <Search size={22} />
              <strong>Inspect any App Store storefront</strong>
              <p>
                Results include release evidence, storefront ratings, and
                batched worldwide estimates.
              </p>
            </div>
          )}
        </div>
      </div>
    </div>,
    document.body,
  );
}

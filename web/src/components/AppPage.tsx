import { useCallback, useEffect, useRef, useState } from "react";
import type { KeyboardEvent } from "react";
import {
  Check,
  ChevronDown,
  Copy,
  ExternalLink,
  Plus,
  RefreshCw,
  Settings2,
  Star,
  Trash2,
  X,
} from "lucide-react";
import { api } from "../api";
import {
  countryFlag,
  countryLabel,
  formatCount,
  isStale,
  relativeTime,
} from "../format";
import type {
  AppSummary,
  AppView,
  Country,
  Keyword,
  Project,
} from "../types";
import KeywordsTable from "./KeywordsTable";
import Overview from "./Overview";
import ReviewsPanel from "./ReviewsPanel";

interface Props {
  project: Project;
  app: AppSummary;
  countries: Country[];
  onAppChanged: () => Promise<void>;
}

type AppTab = "overview" | "keywords" | "reviews";

const appTabs: AppTab[] = ["overview", "keywords", "reviews"];

export default function AppPage({
  project,
  app,
  countries,
  onAppChanged,
}: Props) {
  const [country, setCountry] = useState(app.main_country);
  const [view, setView] = useState<AppView | null>(null);
  const [keywords, setKeywords] = useState<Keyword[]>([]);
  const [tab, setTab] = useState<AppTab>("overview");
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [error, setError] = useState("");
  const [reviewCount, setReviewCount] = useState(0);
  const [storefrontManager, setStorefrontManager] = useState(false);
  const [storefrontCountry, setStorefrontCountry] = useState("us");
  const [storefrontAutoRefresh, setStorefrontAutoRefresh] = useState(false);
  const [storefrontBusy, setStorefrontBusy] = useState(false);
  const [appIdCopied, setAppIdCopied] = useState(false);
  const automaticRefreshes = useRef(new Set<string>());
  const copyReset = useRef<number | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError("");
    try {
      setView(await api.app(project.id, app.apple_id, country));
    } catch (reason) {
      setError((reason as Error).message);
    } finally {
      setLoading(false);
    }
  }, [project.id, app.apple_id, country]);

  useEffect(() => {
    void load();
  }, [load]);

  useEffect(
    () => () => {
      if (copyReset.current != null) window.clearTimeout(copyReset.current);
    },
    [],
  );

  useEffect(() => {
    setReviewCount(view?.review_summary.count ?? 0);
  }, [view?.review_summary.count]);

  useEffect(() => {
    if (!view || !isStale(view.details_updated_at)) return;
    const key = `${project.id}:${app.apple_id}:${country}`;
    if (automaticRefreshes.current.has(key)) return;
    automaticRefreshes.current.add(key);
    setRefreshing(true);
    api
      .refreshApp(project.id, app.apple_id, country)
      .then(setView)
      .catch((reason: Error) => setError(reason.message))
      .finally(() => setRefreshing(false));
  }, [view, project.id, app.apple_id, country]);

  useEffect(() => {
    if (tab !== "keywords") return;
    api
      .keywords(project.id, app.apple_id)
      .then(setKeywords)
      .catch((reason: Error) => setError(reason.message));
  }, [tab, project.id, app.apple_id]);

  const refresh = async () => {
    setRefreshing(true);
    setError("");
    try {
      setView(await api.refreshApp(project.id, app.apple_id, country));
      if (tab === "keywords") {
        setKeywords(await api.keywords(project.id, app.apple_id));
      }
    } catch (reason) {
      setError((reason as Error).message);
    } finally {
      setRefreshing(false);
    }
  };

  const copyAppId = async () => {
    try {
      const appId = String(app.apple_id);
      if (navigator.clipboard && window.isSecureContext) {
        await navigator.clipboard.writeText(appId);
      } else {
        const field = document.createElement("textarea");
        field.value = appId;
        field.style.position = "fixed";
        field.style.opacity = "0";
        document.body.appendChild(field);
        field.select();
        const copied = document.execCommand("copy");
        field.remove();
        if (!copied) throw new Error("clipboard copy failed");
      }
      setAppIdCopied(true);
      if (copyReset.current != null) window.clearTimeout(copyReset.current);
      copyReset.current = window.setTimeout(() => setAppIdCopied(false), 1600);
    } catch {
      setError("Could not copy the App Store ID. Copy it from the header.");
    }
  };

  const reloadStorefronts = async (preferredCountry?: string) => {
    const selected = preferredCountry ?? country;
    const nextView = await api.app(project.id, app.apple_id, selected);
    setView(nextView);
    setCountry(nextView.selected_country);
    await onAppChanged();
  };

  const addStorefront = async (event: React.FormEvent) => {
    event.preventDefault();
    setStorefrontBusy(true);
    setError("");
    try {
      await api.addStorefront(
        project.id,
        app.apple_id,
        storefrontCountry,
        storefrontAutoRefresh,
      );
      await reloadStorefronts(storefrontCountry);
      setStorefrontAutoRefresh(false);
    } catch (reason) {
      setError((reason as Error).message);
    } finally {
      setStorefrontBusy(false);
    }
  };

  const updateStorefront = async (
    storefrontCountryCode: string,
    changes: { is_main?: boolean; auto_refresh?: boolean },
  ) => {
    setStorefrontBusy(true);
    setError("");
    try {
      await api.updateStorefront(
        project.id,
        app.apple_id,
        storefrontCountryCode,
        changes,
      );
      await reloadStorefronts();
    } catch (reason) {
      setError((reason as Error).message);
    } finally {
      setStorefrontBusy(false);
    }
  };

  const deleteStorefront = async (storefrontCountryCode: string) => {
    setStorefrontBusy(true);
    setError("");
    try {
      await api.deleteStorefront(
        project.id,
        app.apple_id,
        storefrontCountryCode,
      );
      await reloadStorefronts();
    } catch (reason) {
      setError((reason as Error).message);
    } finally {
      setStorefrontBusy(false);
    }
  };

  const details = view?.details?.data[0];
  const availableCountries = countries.filter(
    (item) =>
      !view?.storefronts.some((storefront) => storefront.country === item.code),
  );
  const availableCountryCodes = availableCountries
    .map((item) => item.code)
    .join(",");

  useEffect(() => {
    if (!availableCountries.length) return;
    if (!availableCountries.some((item) => item.code === storefrontCountry)) {
      setStorefrontCountry(availableCountries[0].code);
    }
  }, [availableCountryCodes, storefrontCountry]);

  const handleTabKeyDown = (
    event: KeyboardEvent<HTMLButtonElement>,
    currentTab: AppTab,
  ) => {
    const currentIndex = appTabs.indexOf(currentTab);
    let nextIndex: number | null = null;

    if (event.key === "ArrowRight") {
      nextIndex = (currentIndex + 1) % appTabs.length;
    } else if (event.key === "ArrowLeft") {
      nextIndex = (currentIndex - 1 + appTabs.length) % appTabs.length;
    } else if (event.key === "Home") {
      nextIndex = 0;
    } else if (event.key === "End") {
      nextIndex = appTabs.length - 1;
    }

    if (nextIndex == null) return;
    event.preventDefault();
    const nextTab = appTabs[nextIndex];
    setTab(nextTab);
    requestAnimationFrame(() => {
      document.getElementById(`app-tab-${nextTab}`)?.focus();
    });
  };

  return (
    <div className="app-page">
      <header className="app-header">
        <div className="app-identity">
          {details?.icon_url ? (
            <img src={details.icon_url} alt="" />
          ) : (
            <span className="large-placeholder">
              {(details?.name ?? String(app.apple_id)).slice(0, 1)}
            </span>
          )}
          <div>
            <h1>{details?.name ?? app.name ?? app.apple_id}</h1>
            <p>
              <strong>{project.name}</strong>
              <span className="dot">/</span>
              {details?.developer_name ?? "Unknown developer"}
              {details?.primary_category ? (
                <>
                  <span className="dot">/</span>
                  {details.primary_category}
                </>
              ) : null}
              <button
                className={appIdCopied ? "app-id-copy copied" : "app-id-copy"}
                type="button"
                aria-label={`Copy App Store ID ${app.apple_id}`}
                title="Copy App Store ID"
                onClick={() => void copyAppId()}
              >
                {appIdCopied ? <Check size={10} /> : <Copy size={10} />}
                {appIdCopied ? "Copied" : `ID ${app.apple_id}`}
              </button>
            </p>
          </div>
        </div>

        <div className="header-actions">
          <label className="country-select">
            <span>{countryFlag(country)}</span>
            <select
              value={country}
              onChange={(event) => setCountry(event.target.value)}
              disabled={!view}
            >
              {view?.storefronts.map((storefront) => (
                <option key={storefront.country} value={storefront.country}>
                  {countryLabel(storefront.country)}
                  {storefront.is_main ? " · Main" : ""}
                </option>
              ))}
            </select>
            <ChevronDown size={14} />
          </label>
          <button
            className="icon-button"
            onClick={() => void refresh()}
            disabled={refreshing}
            title="Refresh this storefront"
          >
            <RefreshCw className={refreshing ? "spinning" : ""} size={17} />
          </button>
          <button
            className="icon-button"
            onClick={() => setStorefrontManager((visible) => !visible)}
            title="Manage storefronts"
            aria-expanded={storefrontManager}
          >
            {storefrontManager ? <X size={17} /> : <Settings2 size={17} />}
          </button>
          {details?.app_store_url ? (
            <a
              className="icon-button"
              href={details.app_store_url}
              target="_blank"
              rel="noreferrer"
              title="Open in the App Store"
            >
              <ExternalLink size={17} />
            </a>
          ) : null}
        </div>
      </header>

      {storefrontManager && view ? (
        <section className="storefront-manager" aria-label="Storefront settings">
          <form onSubmit={addStorefront}>
            <strong>Add storefront</strong>
            <label>
              <span>Country</span>
              <select
                value={storefrontCountry}
                onChange={(event) => setStorefrontCountry(event.target.value)}
                disabled={!availableCountries.length || storefrontBusy}
              >
                {availableCountries.map((item) => (
                  <option key={item.code} value={item.code}>
                    {item.name}
                  </option>
                ))}
              </select>
            </label>
            <label className="check-control">
              <input
                type="checkbox"
                checked={storefrontAutoRefresh}
                onChange={(event) =>
                  setStorefrontAutoRefresh(event.target.checked)
                }
              />
              Auto refresh
            </label>
            <button
              className="ledger-button"
              type="submit"
              disabled={!availableCountries.length || storefrontBusy}
            >
              <Plus size={13} />
              Add
            </button>
          </form>
          <div className="storefront-records">
            {view.storefronts.map((storefront) => (
              <div key={storefront.country}>
                <strong>
                  {countryFlag(storefront.country)}{" "}
                  {countryLabel(storefront.country)}
                </strong>
                <span>{storefront.is_main ? "Main" : "Additional"}</span>
                <label className="check-control">
                  <input
                    type="checkbox"
                    checked={storefront.auto_refresh}
                    disabled={storefront.is_main || storefrontBusy}
                    onChange={(event) =>
                      void updateStorefront(storefront.country, {
                        auto_refresh: event.target.checked,
                      })
                    }
                  />
                  Auto
                </label>
                {!storefront.is_main ? (
                  <>
                    <button
                      type="button"
                      onClick={() =>
                        void updateStorefront(storefront.country, {
                          is_main: true,
                        })
                      }
                      disabled={storefrontBusy}
                    >
                      Make main
                    </button>
                    <button
                      className="destructive-icon"
                      type="button"
                      title={`Remove ${countryLabel(storefront.country)}`}
                      onClick={() => void deleteStorefront(storefront.country)}
                      disabled={storefrontBusy}
                    >
                      <Trash2 size={12} />
                    </button>
                  </>
                ) : null}
              </div>
            ))}
          </div>
        </section>
      ) : null}

      <div className="app-meta-strip">
        <span>
          <Star size={13} fill="currentColor" />
          <strong>{details?.rating?.toFixed(1) ?? "—"}</strong>
          {formatCount(details?.rating_count)} ratings
        </span>
        <span>
          Version <strong>{details?.version ?? "—"}</strong>
        </span>
        <span>
          Updated{" "}
          <strong>{relativeTime(view?.details_updated_at ?? null)}</strong>
        </span>
        <span className={refreshing ? "sync-state active" : "sync-state"}>
          {refreshing ? "Refreshing" : "Current"}
        </span>
      </div>

      <div className="app-tabs" role="tablist" aria-label="App research">
        <button
          id="app-tab-overview"
          role="tab"
          aria-selected={tab === "overview"}
          aria-controls="app-panel-overview"
          tabIndex={tab === "overview" ? 0 : -1}
          className={tab === "overview" ? "active" : ""}
          onClick={() => setTab("overview")}
          onKeyDown={(event) => handleTabKeyDown(event, "overview")}
        >
          Overview
        </button>
        <button
          id="app-tab-keywords"
          role="tab"
          aria-selected={tab === "keywords"}
          aria-controls="app-panel-keywords"
          tabIndex={tab === "keywords" ? 0 : -1}
          className={tab === "keywords" ? "active" : ""}
          onClick={() => setTab("keywords")}
          onKeyDown={(event) => handleTabKeyDown(event, "keywords")}
        >
          Keywords
          {keywords.length ? <small>{keywords.length}</small> : null}
        </button>
        <button
          id="app-tab-reviews"
          role="tab"
          aria-selected={tab === "reviews"}
          aria-controls="app-panel-reviews"
          tabIndex={tab === "reviews" ? 0 : -1}
          className={tab === "reviews" ? "active" : ""}
          onClick={() => setTab("reviews")}
          onKeyDown={(event) => handleTabKeyDown(event, "reviews")}
        >
          Reviews
          {reviewCount ? (
            <small>{formatCount(reviewCount)}</small>
          ) : null}
        </button>
      </div>

      {error ? <div className="inline-error">{error}</div> : null}

      <div className={tab === "reviews" ? "app-body reviews-mode" : "app-body"}>
        <section
          id={`app-panel-${tab}`}
          className="app-content"
          role="tabpanel"
          aria-labelledby={`app-tab-${tab}`}
          tabIndex={0}
        >
          {loading && !view ? (
            <div className="content-loading">
              <span className="registry-loader" aria-hidden="true">
                <i />
                <i />
                <i />
              </span>
            </div>
          ) : tab === "overview" ? (
            <Overview view={view} />
          ) : tab === "keywords" ? (
            <KeywordsTable
              projectId={project.id}
              appId={app.apple_id}
              keywords={keywords}
              storefronts={view?.storefronts ?? []}
              onChange={setKeywords}
            />
          ) : (
            <ReviewsPanel
              projectId={project.id}
              appId={app.apple_id}
              country={country}
              summary={view?.review_summary}
              onTotalChange={setReviewCount}
            />
          )}
        </section>
      </div>
    </div>
  );
}

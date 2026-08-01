import { useEffect, useMemo, useState } from "react";
import {
  ArrowDown,
  ArrowUp,
  Globe2,
  Minus,
  Plus,
  Trash2,
  TrendingDown,
  TrendingUp,
} from "lucide-react";
import { api } from "../api";
import { countryFlag, countryLabel, relativeTime } from "../format";
import type { Country, Keyword, TrendPoint } from "../types";
import ConfirmDialog from "./ConfirmDialog";
import Picker from "./Picker";
import type { PickerOption } from "./Picker";
import RankingAppButton from "./RankingAppButton";
import RankingResultsDialog from "./RankingResultsDialog";

interface Props {
  projectId: string;
  appId: number;
  keywords: Keyword[];
  countries: Country[];
  defaultCountry: string;
  countryFilter: string;
  onCountryFilterChange: (country: string) => void;
  onChange: (keywords: Keyword[]) => void;
}

type SortKey =
  | "keyword"
  | "country"
  | "difficulty"
  | "popularity"
  | "position";
type SortDirection = "asc" | "desc";

export default function KeywordsTable({
  projectId,
  appId,
  keywords,
  countries,
  defaultCountry,
  countryFilter,
  onCountryFilterChange,
  onChange,
}: Props) {
  const [keyword, setKeyword] = useState("");
  const [country, setCountry] = useState(defaultCountry);
  const [sortKey, setSortKey] = useState<SortKey>("position");
  const [sortDirection, setSortDirection] =
    useState<SortDirection>("asc");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [deleteTarget, setDeleteTarget] = useState<Keyword | null>(null);

  const countryOptions = useMemo<PickerOption[]>(
    () =>
      countries.map((item) => ({
        value: item.code,
        label: item.name,
        triggerLabel: item.code.toUpperCase(),
        meta: item.code.toUpperCase(),
        icon: countryFlag(item.code),
      })),
    [countries],
  );

  const filterOptions = useMemo<PickerOption[]>(
    () => {
      const tracked = new Set(keywords.map((item) => item.country));
      return [
        {
          value: "all",
          label: "All storefronts",
          triggerLabel: "All",
          meta: `${tracked.size} tracked`,
          icon: <Globe2 size={13} />,
        },
        ...countryOptions.filter((option) => tracked.has(option.value)),
      ];
    },
    [countryOptions, keywords],
  );

  useEffect(() => {
    if (
      countryFilter !== "all" &&
      !keywords.some((item) => item.country === countryFilter)
    ) {
      onCountryFilterChange("all");
    }
  }, [countryFilter, keywords, onCountryFilterChange]);

  const sorted = useMemo(() => {
    const items =
      countryFilter === "all"
        ? [...keywords]
        : keywords.filter((item) => item.country === countryFilter);
    const direction = sortDirection === "asc" ? 1 : -1;
    return items.sort((left, right) => {
      let comparison = 0;
      if (sortKey === "keyword") {
        comparison = left.keyword.localeCompare(right.keyword, undefined, {
          sensitivity: "base",
        });
      } else if (sortKey === "country") {
        comparison = left.country.localeCompare(right.country);
      } else if (sortKey === "difficulty" || sortKey === "popularity") {
        const leftScore = left[sortKey];
        const rightScore = right[sortKey];
        if (leftScore == null && rightScore != null) return 1;
        if (leftScore != null && rightScore == null) return -1;
        comparison = (leftScore ?? 0) - (rightScore ?? 0);
      } else {
        if (left.position == null && right.position != null) return 1;
        if (left.position != null && right.position == null) return -1;
        comparison =
          (left.position ?? 0) - (right.position ?? 0);
      }
      if (comparison === 0) {
        comparison = left.keyword.localeCompare(right.keyword, undefined, {
          sensitivity: "base",
        });
      }
      if (comparison === 0) comparison = left.country.localeCompare(right.country);
      return comparison * direction;
    });
  }, [countryFilter, keywords, sortDirection, sortKey]);

  const changeSort = (nextKey: SortKey) => {
    if (sortKey === nextKey) {
      setSortDirection((current) => (current === "asc" ? "desc" : "asc"));
    } else {
      setSortKey(nextKey);
      setSortDirection("asc");
    }
  };

  const submit = async (event: React.FormEvent) => {
    event.preventDefault();
    if (!keyword.trim()) return;
    setBusy(true);
    setError("");
    try {
      await api.addKeyword(projectId, keyword.trim(), country);
      onChange(await api.keywords(projectId, appId));
      setKeyword("");
    } catch (reason) {
      setError((reason as Error).message);
    } finally {
      setBusy(false);
    }
  };

  const remove = async () => {
    if (!deleteTarget) return;
    setBusy(true);
    setError("");
    try {
      await api.deleteKeyword(projectId, deleteTarget.query_id);
      onChange(
        keywords.filter((item) => item.query_id !== deleteTarget.query_id),
      );
      setDeleteTarget(null);
    } catch (reason) {
      setError((reason as Error).message);
    } finally {
      setBusy(false);
    }
  };

  const removeFromAllStorefronts = async () => {
    if (!deleteTarget) return;
    const normalized = deleteTarget.normalized_keyword;
    const matches = keywords.filter(
      (item) => item.normalized_keyword === normalized,
    );
    setBusy(true);
    setError("");
    try {
      const results = await Promise.allSettled(
        matches.map((item) =>
          api.deleteKeyword(projectId, item.query_id),
        ),
      );
      const removedIds = new Set(
        matches
          .filter((_, index) => results[index].status === "fulfilled")
          .map((item) => item.query_id),
      );
      onChange(keywords.filter((item) => !removedIds.has(item.query_id)));
      setDeleteTarget(null);
      const failed = matches.length - removedIds.size;
      if (failed) {
        setError(
          `Removed “${deleteTarget.keyword}” from ${removedIds.size} storefront${
            removedIds.size === 1 ? "" : "s"
          }; ${failed} failed. Retry the remaining entries.`,
        );
      }
    } catch (reason) {
      setError((reason as Error).message);
    } finally {
      setBusy(false);
    }
  };

  const addToStorefront = async (source: Keyword, selection: string) => {
    if (!selection) return;
    setBusy(true);
    setError("");
    try {
      await api.addKeyword(projectId, source.keyword, selection);
      onChange(await api.keywords(projectId, appId));
    } catch (reason) {
      setError((reason as Error).message);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="keywords-panel">
      <div className="keyword-toolbar">
        <form className="keyword-create" onSubmit={submit}>
          <strong>Project keyword</strong>
          <input
            value={keyword}
            onChange={(event) => setKeyword(event.target.value)}
            placeholder="Keyword"
            maxLength={200}
          />
          <Picker
            value={country}
            options={countryOptions}
            onChange={setCountry}
            ariaLabel="Keyword storefront"
            className="keyword-country-picker"
            disabled={busy}
            searchPlaceholder="Search storefronts"
          />
          <button type="submit" disabled={busy || !keyword.trim()}>
            <Plus size={13} />
            Add
          </button>
        </form>
        <div className="tab-storefront-filter">
          <span>Show</span>
          <Picker
            value={countryFilter}
            options={filterOptions}
            onChange={onCountryFilterChange}
            ariaLabel="Filter keywords by storefront"
            className="tab-country-picker"
            searchPlaceholder="Search storefronts"
          />
        </div>
      </div>

      {error ? (
        <div className="keyword-form-error" role="alert">
          {error}
        </div>
      ) : null}

      {keywords.length ? (
        sorted.length ? (
          <div className="keyword-table-wrap">
            <table className="keyword-table">
              <caption>
                Project keywords · rank shown for selected app · color favors
                high popularity and low difficulty
              </caption>
              <thead>
                <tr>
                  <SortableHeading
                    label="Keyword"
                    column="keyword"
                    activeColumn={sortKey}
                    direction={sortDirection}
                    onSort={changeSort}
                  />
                  <th>Last update</th>
                  <SortableHeading
                    label="Store"
                    column="country"
                    activeColumn={sortKey}
                    direction={sortDirection}
                    onSort={changeSort}
                  />
                  <SortableHeading
                    label="Position"
                    column="position"
                    activeColumn={sortKey}
                    direction={sortDirection}
                    onSort={changeSort}
                    number
                  />
                  <SortableHeading
                    label="Popularity"
                    column="popularity"
                    activeColumn={sortKey}
                    direction={sortDirection}
                    onSort={changeSort}
                    number
                  />
                  <SortableHeading
                    label="Difficulty"
                    column="difficulty"
                    activeColumn={sortKey}
                    direction={sortDirection}
                    onSort={changeSort}
                    number
                  />
                  <th>Trend</th>
                  <th>Apps in ranking</th>
                  <th aria-label="Actions" />
                </tr>
              </thead>
              <tbody>
                {sorted.map((item) => {
                  const normalized = item.normalized_keyword;
                  const trackedCountries = new Set(
                    keywords
                      .filter(
                        (candidate) =>
                          candidate.normalized_keyword === normalized,
                      )
                      .map((candidate) => candidate.country),
                  );
                  const available = countries
                    .map((item) => item.code)
                    .filter((code) => !trackedCountries.has(code));
                  return (
                    <KeywordRow
                      keyword={item}
                      key={item.query_id}
                      disabled={busy}
                      availableCountries={available}
                      onAddToStorefront={(selection) =>
                        void addToStorefront(item, selection)
                      }
                      onDelete={() => setDeleteTarget(item)}
                      onRefreshRanking={async () => {
                        const refreshed = await api.refreshKeywords(
                          projectId,
                          appId,
                          item.query_id,
                        );
                        onChange(refreshed);
                      }}
                    />
                  );
                })}
              </tbody>
            </table>
          </div>
        ) : (
          <div className="keyword-empty compact-filter-empty">
            <h2>No project keywords in this storefront.</h2>
            <p>Choose All or add a project keyword for this storefront.</p>
          </div>
        )
      ) : (
        <div className="keyword-empty">
          <h2>No project keywords yet.</h2>
          <p>
            Add one above. It will appear in every tracked app with that app’s
            rank and trend.
          </p>
        </div>
      )}

      {deleteTarget ? (
        <ConfirmDialog
          title="Delete project keyword?"
          description={`“${deleteTarget.keyword}” will be removed from the project in ${countryLabel(
            deleteTarget.country,
          )}. It will disappear from every app; its other storefronts stay tracked.`}
          confirmLabel={`Delete ${deleteTarget.country.toUpperCase()} keyword`}
          alternateLabel={
            keywords.filter(
              (item) =>
                item.normalized_keyword === deleteTarget.normalized_keyword,
            ).length > 1
              ? "Delete all storefronts"
              : undefined
          }
          busy={busy}
          onCancel={() => {
            if (!busy) setDeleteTarget(null);
          }}
          onConfirm={() => void remove()}
          onAlternateConfirm={() => void removeFromAllStorefronts()}
        />
      ) : null}
    </div>
  );
}

function SortableHeading({
  label,
  column,
  activeColumn,
  direction,
  onSort,
  number = false,
}: {
  label: string;
  column: SortKey;
  activeColumn: SortKey;
  direction: SortDirection;
  onSort: (column: SortKey) => void;
  number?: boolean;
}) {
  const active = activeColumn === column;
  return (
    <th
      className={number ? "number" : undefined}
      aria-sort={
        active
          ? direction === "asc"
            ? "ascending"
            : "descending"
          : "none"
      }
    >
      <button
        type="button"
        className={active ? "sort-heading active" : "sort-heading"}
        onClick={() => onSort(column)}
      >
        <span>{label}</span>
        {active ? (
          direction === "asc" ? (
            <ArrowUp size={12} />
          ) : (
            <ArrowDown size={12} />
          )
        ) : (
          <span className="sort-placeholder" aria-hidden="true" />
        )}
      </button>
    </th>
  );
}

function KeywordRow({
  keyword,
  disabled,
  availableCountries,
  onAddToStorefront,
  onDelete,
  onRefreshRanking,
}: {
  keyword: Keyword;
  disabled: boolean;
  availableCountries: string[];
  onAddToStorefront: (selection: string) => void;
  onDelete: () => void;
  onRefreshRanking: () => Promise<void>;
}) {
  const movement =
    keyword.position != null && keyword.previous_position != null
      ? keyword.previous_position - keyword.position
      : 0;
  const addOptions: PickerOption[] = [
    ...availableCountries.map((code) => ({
      value: code,
      label: countryLabel(code),
      meta: code.toUpperCase(),
      icon: countryFlag(code),
    })),
  ];

  return (
    <tr>
      <td>
        <strong className="keyword-name" title={keyword.keyword}>
          {keyword.keyword}
        </strong>
      </td>
      <td className="muted">{relativeTime(keyword.last_updated)}</td>
      <td>
        <span className="store-cell" title={countryLabel(keyword.country)}>
          {countryFlag(keyword.country)}
          {keyword.country.toUpperCase()}
        </span>
      </td>
      <td className="number position-cell">
        {keyword.position == null ? (
          <span className="not-ranked">Not ranked</span>
        ) : (
          <>
            <small>#</small>
            <strong>{keyword.position}</strong>
          </>
        )}
      </td>
      <KeywordMetricCell label="Popularity" value={keyword.popularity} />
      <KeywordMetricCell label="Difficulty" value={keyword.difficulty} />
      <td>
        <div className="trend-cell">
          <span
            className={
              movement > 0 ? "up" : movement < 0 ? "down" : "unchanged"
            }
          >
            {movement > 0 ? (
              <TrendingUp size={13} />
            ) : movement < 0 ? (
              <TrendingDown size={13} />
            ) : (
              <Minus size={13} />
            )}
            {movement === 0 ? "0" : Math.abs(movement)}
          </span>
          <Sparkline points={keyword.trend} />
        </div>
      </td>
      <td>
        <RankingAppsCell
          keyword={keyword}
          onRefreshRanking={onRefreshRanking}
        />
      </td>
      <td className="keyword-actions">
        <div>
          <Picker
            value={null}
            options={addOptions}
            onChange={onAddToStorefront}
            ariaLabel={
              availableCountries.length
                ? `Add project keyword ${keyword.keyword} to another storefront`
                : `${keyword.keyword} is tracked in every App Store storefront`
            }
            triggerContent={<Plus size={13} />}
            iconOnly
            disabled={disabled || !availableCountries.length}
            searchPlaceholder="Search storefronts"
          />
          <button
            type="button"
            title={`Delete ${keyword.keyword} from ${countryLabel(
              keyword.country,
            )}`}
            aria-label={`Delete ${keyword.keyword} from ${countryLabel(
              keyword.country,
            )}`}
            disabled={disabled}
            onClick={onDelete}
          >
            <Trash2 size={13} />
          </button>
        </div>
      </td>
    </tr>
  );
}

function KeywordMetricCell({
  label,
  value,
}: {
  label: "Difficulty" | "Popularity";
  value: number | null;
}) {
  const tone =
    value == null
      ? null
      : label === "Popularity"
        ? value <= 20
          ? "unfavorable"
          : value <= 60
            ? "neutral"
            : "favorable"
        : value <= 20
          ? "favorable"
          : value <= 60
            ? "neutral"
            : "unfavorable";
  const interpretation =
    tone === "favorable"
      ? label === "Popularity"
        ? "high popularity"
        : "low difficulty"
      : tone === "unfavorable"
        ? label === "Popularity"
          ? "low popularity"
          : "high difficulty"
        : `medium ${label.toLowerCase()}`;

  return (
    <td className="number keyword-metric-cell">
      {value == null ? (
        <span
          className="metric-unavailable"
          title={`${label} not provided`}
          aria-label={`${label}: not provided`}
        >
          —
        </span>
      ) : (
        <span
          className={`keyword-metric ${tone}`}
          title={`${label}: ${value} out of 100 · ${interpretation}`}
          role="meter"
          aria-label={label}
          aria-valuemin={0}
          aria-valuemax={100}
          aria-valuenow={value}
          aria-valuetext={`${value} out of 100, ${interpretation}`}
        >
          <strong aria-hidden="true">
            {Number.isInteger(value) ? value : value.toFixed(1)}
          </strong>
          <span className="keyword-metric-track" aria-hidden="true">
            <span
              className="keyword-metric-fill"
              style={{ width: `${value}%` }}
            />
          </span>
        </span>
      )}
    </td>
  );
}

function RankingAppsCell({
  keyword,
  onRefreshRanking,
}: {
  keyword: Keyword;
  onRefreshRanking: () => Promise<void>;
}) {
  const [showAll, setShowAll] = useState(false);
  return (
    <>
      <div className="ranking-apps">
        {keyword.apps_in_ranking.slice(0, 5).map((app) => (
          <RankingAppButton
            app={app}
            country={keyword.country}
            keyword={keyword.keyword}
            key={app.apple_id}
          />
        ))}
        {keyword.apps_in_ranking.length > 5 ? (
          <button
            type="button"
            className="ranking-apps-all"
            onClick={() => setShowAll(true)}
            aria-label={`Show all ${keyword.apps_in_ranking.length} ranked apps for ${keyword.keyword}`}
          >
            All {keyword.apps_in_ranking.length}
          </button>
        ) : null}
      </div>
      {showAll ? (
        <RankingResultsDialog
          apps={keyword.apps_in_ranking}
          country={keyword.country}
          keyword={keyword.keyword}
          onRefresh={onRefreshRanking}
          onClose={() => setShowAll(false)}
        />
      ) : null}
    </>
  );
}

function Sparkline({ points }: { points: TrendPoint[] }) {
  const positions = points
    .map((point) => point.position)
    .filter((position): position is number => position != null);
  if (positions.length < 2) {
    return <span className="sparkline-empty" />;
  }
  const min = Math.min(...positions);
  const max = Math.max(...positions);
  const range = Math.max(1, max - min);
  const path = positions
    .map((position, index) => {
      const x = (index / (positions.length - 1)) * 88;
      const y = 4 + ((position - min) / range) * 20;
      return `${index ? "L" : "M"}${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");
  return (
    <svg className="sparkline" viewBox="0 0 88 28" aria-hidden="true">
      <path d={path} />
    </svg>
  );
}

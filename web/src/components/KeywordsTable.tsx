import { useMemo, useState } from "react";
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
import type { Keyword, Storefront, TrendPoint } from "../types";
import ConfirmDialog from "./ConfirmDialog";
import Picker from "./Picker";
import type { PickerOption } from "./Picker";
import RankingAppButton from "./RankingAppButton";
import RankingResultsDialog from "./RankingResultsDialog";

interface Props {
  projectId: string;
  appId: number;
  keywords: Keyword[];
  storefronts: Storefront[];
  countryFilter: string;
  onCountryFilterChange: (country: string) => void;
  onChange: (keywords: Keyword[]) => void;
}

type SortKey = "keyword" | "country" | "position";
type SortDirection = "asc" | "desc";

export default function KeywordsTable({
  projectId,
  appId,
  keywords,
  storefronts,
  countryFilter,
  onCountryFilterChange,
  onChange,
}: Props) {
  const [keyword, setKeyword] = useState("");
  const [country, setCountry] = useState(
    storefronts.find((storefront) => storefront.is_main)?.country ??
      storefronts[0]?.country ??
      "us",
  );
  const [sortKey, setSortKey] = useState<SortKey>("position");
  const [sortDirection, setSortDirection] =
    useState<SortDirection>("asc");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [deleteTarget, setDeleteTarget] = useState<Keyword | null>(null);

  const storefrontOptions = useMemo<PickerOption[]>(
    () =>
      storefronts.map((storefront) => ({
        value: storefront.country,
        label: countryLabel(storefront.country),
        triggerLabel: storefront.country.toUpperCase(),
        meta: `${storefront.country.toUpperCase()}${
          storefront.is_main ? " · Main" : ""
        }`,
        icon: countryFlag(storefront.country),
      })),
    [storefronts],
  );

  const filterOptions = useMemo<PickerOption[]>(
    () => [
      {
        value: "all",
        label: "All storefronts",
        triggerLabel: "All",
        meta: `${storefronts.length} configured`,
        icon: <Globe2 size={13} />,
      },
      ...storefrontOptions,
    ],
    [storefrontOptions, storefronts.length],
  );

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
      } else {
        comparison =
          (left.position ?? Number.MAX_SAFE_INTEGER) -
          (right.position ?? Number.MAX_SAFE_INTEGER);
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
      const created = await api.addKeyword(
        projectId,
        appId,
        keyword.trim(),
        country,
      );
      onChange([...keywords, created]);
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
      await api.deleteKeyword(projectId, appId, deleteTarget.query_id);
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
    const normalized = deleteTarget.keyword.trim().toLocaleLowerCase();
    const matches = keywords.filter(
      (item) => item.keyword.trim().toLocaleLowerCase() === normalized,
    );
    setBusy(true);
    setError("");
    try {
      const results = await Promise.allSettled(
        matches.map((item) =>
          api.deleteKeyword(projectId, appId, item.query_id),
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

  const addToStorefronts = async (
    source: Keyword,
    selection: string,
    available: string[],
  ) => {
    const targets = selection === "__all__" ? available : [selection];
    if (!targets.length) return;
    setBusy(true);
    setError("");
    try {
      const results = await Promise.allSettled(
        targets.map((targetCountry) =>
          api.addKeyword(
            projectId,
            appId,
            source.keyword,
            targetCountry,
          ),
        ),
      );
      const created = results.flatMap((result) =>
        result.status === "fulfilled" ? [result.value] : [],
      );
      const createdIds = new Set(created.map((item) => item.query_id));
      if (created.length) {
        onChange([
          ...keywords.filter((item) => !createdIds.has(item.query_id)),
          ...created,
        ]);
      }
      const failed = results.length - created.length;
      if (failed) {
        setError(
          `Added “${source.keyword}” to ${created.length} storefront${
            created.length === 1 ? "" : "s"
          }; ${failed} failed. Retry the remaining storefronts.`,
        );
      }
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
          <strong>Add keyword</strong>
          <input
            value={keyword}
            onChange={(event) => setKeyword(event.target.value)}
            placeholder="Keyword"
            maxLength={200}
          />
          <Picker
            value={country}
            options={storefrontOptions}
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
                  <th>Trend</th>
                  <th>Apps in ranking</th>
                  <th aria-label="Actions" />
                </tr>
              </thead>
              <tbody>
                {sorted.map((item) => {
                  const normalized = item.keyword.trim().toLocaleLowerCase();
                  const trackedCountries = new Set(
                    keywords
                      .filter(
                        (candidate) =>
                          candidate.keyword.trim().toLocaleLowerCase() ===
                          normalized,
                      )
                      .map((candidate) => candidate.country),
                  );
                  const available = storefronts
                    .map((storefront) => storefront.country)
                    .filter((code) => !trackedCountries.has(code));
                  return (
                    <KeywordRow
                      keyword={item}
                      key={item.query_id}
                      disabled={busy}
                      availableCountries={available}
                      onAddToStorefront={(selection) =>
                        void addToStorefronts(item, selection, available)
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
            <h2>No keywords in this storefront.</h2>
            <p>Choose All or add a storefront-specific query above.</p>
          </div>
        )
      ) : (
        <div className="keyword-empty">
          <h2>No keywords yet.</h2>
          <p>
            Add a storefront-specific query above. Shared query results are
            cached once per project.
          </p>
        </div>
      )}

      {deleteTarget ? (
        <ConfirmDialog
          title="Delete tracked keyword?"
          description={`“${deleteTarget.keyword}” will be removed from ${countryLabel(
            deleteTarget.country,
          )}. Its other storefronts stay tracked.`}
          confirmLabel={`Delete from ${deleteTarget.country.toUpperCase()}`}
          alternateLabel={
            keywords.filter(
              (item) =>
                item.keyword.trim().toLocaleLowerCase() ===
                deleteTarget.keyword.trim().toLocaleLowerCase(),
            ).length > 1
              ? "Delete from all"
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
    ...(availableCountries.length > 1
      ? [
          {
            value: "__all__",
            label: "All remaining storefronts",
            meta: `${availableCountries.length} storefronts`,
            icon: <Globe2 size={13} />,
          },
        ]
      : []),
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
                ? `Add ${keyword.keyword} to another storefront`
                : `${keyword.keyword} is tracked in every storefront`
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

import { useMemo, useState } from "react";
import { Minus, Plus, Trash2, TrendingDown, TrendingUp } from "lucide-react";
import { api } from "../api";
import { countryFlag, countryLabel, relativeTime } from "../format";
import type { Keyword, Storefront, TrendPoint } from "../types";

interface Props {
  projectId: string;
  appId: number;
  keywords: Keyword[];
  storefronts: Storefront[];
  onChange: (keywords: Keyword[]) => void;
}

export default function KeywordsTable({
  projectId,
  appId,
  keywords,
  storefronts,
  onChange,
}: Props) {
  const [keyword, setKeyword] = useState("");
  const [notes, setNotes] = useState("");
  const [country, setCountry] = useState(
    storefronts.find((storefront) => storefront.is_main)?.country ??
      storefronts[0]?.country ??
      "us",
  );
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const sorted = useMemo(
    () =>
      [...keywords].sort(
        (left, right) =>
          (left.position ?? Number.MAX_SAFE_INTEGER) -
          (right.position ?? Number.MAX_SAFE_INTEGER),
      ),
    [keywords],
  );

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
        notes.trim(),
      );
      onChange([...keywords, created]);
      setKeyword("");
      setNotes("");
    } catch (reason) {
      setError((reason as Error).message);
    } finally {
      setBusy(false);
    }
  };

  const remove = async (queryId: number) => {
    setBusy(true);
    setError("");
    try {
      await api.deleteKeyword(projectId, appId, queryId);
      onChange(keywords.filter((item) => item.query_id !== queryId));
    } catch (reason) {
      setError((reason as Error).message);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="keywords-panel">
      <form className="keyword-create" onSubmit={submit}>
        <strong>Add keyword</strong>
        <input
          value={keyword}
          onChange={(event) => setKeyword(event.target.value)}
          placeholder="Keyword"
          maxLength={200}
        />
        <input
          value={notes}
          onChange={(event) => setNotes(event.target.value)}
          placeholder="Notes (optional)"
          maxLength={500}
        />
        <select
          value={country}
          onChange={(event) => setCountry(event.target.value)}
          aria-label="Keyword storefront"
        >
          {storefronts.map((storefront) => (
            <option value={storefront.country} key={storefront.country}>
              {countryLabel(storefront.country)}
            </option>
          ))}
        </select>
        <button type="submit" disabled={busy || !keyword.trim()}>
          <Plus size={13} />
          Add
        </button>
      </form>
      {error ? (
        <div className="keyword-form-error" role="alert">
          {error}
        </div>
      ) : null}
      {keywords.length ? (
        <div className="keyword-table-wrap">
          <table className="keyword-table">
            <thead>
              <tr>
                <th>Keyword</th>
                <th>Notes</th>
                <th>Last update</th>
                <th>Store</th>
                <th className="number">Position</th>
                <th>Trend</th>
                <th>Apps in ranking</th>
                <th aria-label="Actions" />
              </tr>
            </thead>
            <tbody>
              {sorted.map((item) => (
                <KeywordRow
                  keyword={item}
                  key={item.query_id}
                  disabled={busy}
                  onDelete={() => void remove(item.query_id)}
                />
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <div className="keyword-empty">
          <h2>No keywords yet.</h2>
          <p>
            Add a storefront-specific query above. Shared query results are
            cached once per project.
          </p>
        </div>
      )}
    </div>
  );
}

function KeywordRow({
  keyword,
  disabled,
  onDelete,
}: {
  keyword: Keyword;
  disabled: boolean;
  onDelete: () => void;
}) {
  const movement =
    keyword.position != null && keyword.previous_position != null
      ? keyword.previous_position - keyword.position
      : 0;
  return (
    <tr>
      <td>
        <strong className="keyword-name" title={keyword.keyword}>
          {keyword.keyword}
        </strong>
      </td>
      <td>
        <span className="keyword-note" title={keyword.notes}>
          {keyword.notes || "—"}
        </span>
      </td>
      <td className="muted">{relativeTime(keyword.last_updated)}</td>
      <td>
        <span className="store-cell" title={countryLabel(keyword.country)}>
          {countryFlag(keyword.country)}
          {countryLabel(keyword.country)}
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
        <div className="ranking-apps">
          {keyword.apps_in_ranking.map((app) =>
            app.icon_url ? (
              <img
                key={app.apple_id}
                src={app.icon_url}
                title={`#${app.position} ${app.name}`}
                alt={app.name}
                loading="lazy"
              />
            ) : (
              <span
                key={app.apple_id}
                className="rank-placeholder"
                title={`#${app.position} ${app.name}`}
              >
                {app.name.slice(0, 1)}
              </span>
            ),
          )}
        </div>
      </td>
      <td className="keyword-actions">
        <button
          type="button"
          title={`Remove ${keyword.keyword}`}
          disabled={disabled}
          onClick={onDelete}
        >
          <Trash2 size={13} />
        </button>
      </td>
    </tr>
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

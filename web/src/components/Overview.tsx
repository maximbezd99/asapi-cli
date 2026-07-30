import { ArrowUpRight, Globe2, Star } from "lucide-react";
import {
  countryFlag,
  countryLabel,
  formatCount,
  formatDate,
  relativeTime,
} from "../format";
import type { AppView } from "../types";

export default function Overview({ view }: { view: AppView | null }) {
  const details = view?.details?.data[0];
  if (!view || !details) {
    return (
      <div className="empty-section">
        No storefront data has been fetched yet.
      </div>
    );
  }
  const popularity = view.popularity?.countries ?? [];
  const purchases =
    details.in_app_purchases ?? view.iap?.data.purchases ?? [];
  const similar = details.similar_apps ?? view.similar?.data ?? [];

  return (
    <div className="overview">
      {details.screenshots?.length ? (
        <section className="panel screenshots-panel">
          <div className="section-heading">
            <h2>Screenshots</h2>
            <span>{details.screenshots.length}</span>
          </div>
          <div className="screenshots">
            {details.screenshots.map((screenshot) => (
              <img src={screenshot} alt="" key={screenshot} loading="lazy" />
            ))}
          </div>
        </section>
      ) : null}

      <section className="panel">
        <div className="section-heading">
          <h2>Information</h2>
          <span>{details.content_rating ?? "—"}</span>
        </div>
        <div className="facts">
          <Fact label="Seller" value={details.seller_name} />
          <Fact label="Released" value={formatDate(details.released_at)} />
          <Fact
            label="Last update"
            value={formatDate(details.version_released_at)}
          />
          <Fact label="Minimum OS" value={details.minimum_os_version} />
          <Fact
            label="Size"
            value={
              details.size_bytes
                ? `${(details.size_bytes / 1_000_000).toFixed(1)} MB`
                : undefined
            }
          />
          <Fact label="Languages" value={details.languages?.join(", ")} wide />
        </div>
      </section>

      <section className="panel popularity-panel">
        <div className="section-heading">
          <h2>Popularity by storefront</h2>
          <span>
            <Globe2 size={14} />
            {view.popularity
              ? `Updated ${relativeTime(view.popularity.fetched_at)}`
              : "Not fetched"}
          </span>
        </div>
        {popularity.length ? (
          <div className="popularity-grid">
            {popularity.map((item) => (
              <article
                key={item.country}
                className={item.available ? "" : "unavailable"}
              >
                <span className="popularity-country">
                  {countryFlag(item.country)}
                  <strong>{countryLabel(item.country)}</strong>
                  <small>{item.country.toUpperCase()}</small>
                </span>
                <span className="popularity-rating">
                    <Star size={12} fill="currentColor" />
                    {item.rating?.toFixed(1) ?? "—"}
                </span>
                <strong className="popularity-count">
                  {item.available ? formatCount(item.rating_count) : "—"}
                </strong>
              </article>
            ))}
          </div>
        ) : (
          <div className="empty-section">Popularity has not been fetched.</div>
        )}
      </section>

      {details.release_notes ? (
        <section className="panel">
        <div className="section-heading">
          <h2>What’s new</h2>
          <span>
            v{details.version} / {formatDate(details.version_released_at)}
          </span>
          </div>
          <p className="description">{details.release_notes}</p>
        </section>
      ) : null}

      <div className="two-column">
        <section className="panel">
          <div className="section-heading">
            <h2>In-app purchases</h2>
            <span>{purchases.length}</span>
          </div>
          {purchases.length ? (
            <div className="purchase-list">
              {purchases.map((purchase, index) => (
                <div key={`${purchase.name}:${index}`}>
                  <span>{purchase.name}</span>
                  <strong>{purchase.display_price}</strong>
                </div>
              ))}
            </div>
          ) : (
            <div className="empty-section compact">
              No displayed in-app purchases.
            </div>
          )}
        </section>

        <section className="panel">
          <div className="section-heading">
            <h2>Similar apps</h2>
            <span>{similar.length}</span>
          </div>
          <div className="similar-list">
            {similar.map((app) => (
              <div key={app.app_id}>
                {app.icon_url ? (
                  <img src={app.icon_url} alt="" loading="lazy" />
                ) : (
                  <span className="mini-placeholder">{app.name.slice(0, 1)}</span>
                )}
                <span>
                  <strong>{app.name}</strong>
                  <small>{app.developer_name ?? "Unknown developer"}</small>
                </span>
                <ArrowUpRight size={14} />
              </div>
            ))}
            {!similar.length ? (
              <div className="empty-section compact">No similar apps found.</div>
            ) : null}
          </div>
        </section>
      </div>

      <section className="panel about-panel">
        <div className="section-heading">
          <h2>About</h2>
        </div>
        <p className="description">
          {details.description ?? "No description is available."}
        </p>
      </section>
    </div>
  );
}

function Fact({
  label,
  value,
  wide,
}: {
  label: string;
  value?: string | null;
  wide?: boolean;
}) {
  return (
    <div className={wide ? "wide" : ""}>
      <small>{label}</small>
      <strong>{value || "—"}</strong>
    </div>
  );
}

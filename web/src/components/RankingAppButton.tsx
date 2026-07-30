import { useEffect, useId, useRef, useState } from "react";
import { ExternalLink, Star, X } from "lucide-react";
import { createPortal } from "react-dom";
import { api } from "../api";
import { countryFlag, countryLabel, formatCount } from "../format";
import type { AppDetails, RankedApp } from "../types";

interface Props {
  app: RankedApp;
  country: string;
  keyword: string;
}

interface Point {
  x: number;
  y: number;
}

function cursorPoint(clientX: number, clientY: number): Point {
  const tooltipWidth = 230;
  const x =
    clientX + 14 + tooltipWidth > window.innerWidth
      ? clientX - tooltipWidth - 14
      : clientX + 14;
  const y = Math.min(clientY + 14, window.innerHeight - 48);
  return { x: Math.max(8, x), y: Math.max(8, y) };
}

export default function RankingAppButton({ app, country, keyword }: Props) {
  const [tooltip, setTooltip] = useState<Point | null>(null);
  const [cardPosition, setCardPosition] = useState<Point | null>(null);
  const [details, setDetails] = useState<AppDetails | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const buttonRef = useRef<HTMLButtonElement | null>(null);
  const cardRef = useRef<HTMLDivElement | null>(null);
  const cardId = useId();

  const openCard = () => {
    const rect = buttonRef.current?.getBoundingClientRect();
    if (!rect) return;
    const width = Math.min(310, window.innerWidth - 16);
    const estimatedHeight = 220;
    const left = Math.max(
      8,
      Math.min(rect.left, window.innerWidth - width - 8),
    );
    const top =
      rect.bottom + estimatedHeight + 8 < window.innerHeight
        ? rect.bottom + 5
        : Math.max(8, rect.top - estimatedHeight - 5);
    setTooltip(null);
    setCardPosition({ x: left, y: top });
  };

  const closeCard = (restoreFocus = false) => {
    setCardPosition(null);
    if (restoreFocus) requestAnimationFrame(() => buttonRef.current?.focus());
  };

  useEffect(() => {
    if (!cardPosition || details || loading) return;
    let cancelled = false;
    setLoading(true);
    setError("");
    api
      .lookupApp(app.apple_id, country)
      .then((result) => {
        if (cancelled) return;
        if (result) {
          setDetails(result);
        } else {
          setError("The app is not available in this storefront.");
        }
      })
      .catch((reason: Error) => {
        if (!cancelled) setError(reason.message);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [app.apple_id, cardPosition, country, details]);

  useEffect(() => {
    if (!cardPosition) return;
    cardRef.current?.focus();
    const handlePointerDown = (event: PointerEvent) => {
      const target = event.target as Node;
      if (
        cardRef.current?.contains(target) ||
        buttonRef.current?.contains(target)
      ) {
        return;
      }
      closeCard(true);
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        closeCard(true);
      }
    };
    document.addEventListener("pointerdown", handlePointerDown, true);
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("pointerdown", handlePointerDown, true);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [cardPosition]);

  const appStoreUrl =
    details?.app_store_url ??
    `https://apps.apple.com/${country}/app/id${app.apple_id}`;

  return (
    <>
      <button
        ref={buttonRef}
        type="button"
        className="ranking-app-button"
        aria-label={`Open details for #${app.position} ${app.name}`}
        aria-expanded={Boolean(cardPosition)}
        aria-controls={cardPosition ? cardId : undefined}
        onPointerEnter={(event) =>
          setTooltip(cursorPoint(event.clientX, event.clientY))
        }
        onPointerMove={(event) =>
          setTooltip(cursorPoint(event.clientX, event.clientY))
        }
        onPointerLeave={() => setTooltip(null)}
        onFocus={() => {
          const rect = buttonRef.current?.getBoundingClientRect();
          if (rect) setTooltip(cursorPoint(rect.right, rect.top));
        }}
        onBlur={() => setTooltip(null)}
        onClick={() => (cardPosition ? closeCard() : openCard())}
      >
        {app.icon_url ? (
          <img src={app.icon_url} alt="" loading="lazy" />
        ) : (
          <span className="rank-placeholder" aria-hidden="true">
            {app.name.slice(0, 1)}
          </span>
        )}
      </button>

      {tooltip && !cardPosition
        ? createPortal(
            <div
              className="ranking-tooltip"
              role="tooltip"
              style={{ left: tooltip.x, top: tooltip.y }}
            >
              <strong>{app.name}</strong>
              <span>
                #{app.position} · {app.developer_name ?? "Unknown developer"}
              </span>
            </div>,
            document.body,
          )
        : null}

      {cardPosition
        ? createPortal(
            <div
              ref={cardRef}
              id={cardId}
              className="ranking-card"
              role="dialog"
              aria-label={`App details for ${app.name}`}
              tabIndex={-1}
              style={{ left: cardPosition.x, top: cardPosition.y }}
            >
              <header>
                {details?.icon_url || app.icon_url ? (
                  <img src={details?.icon_url ?? app.icon_url ?? ""} alt="" />
                ) : (
                  <span className="ranking-card-placeholder">
                    {app.name.slice(0, 1)}
                  </span>
                )}
                <span>
                  <strong>{details?.name ?? app.name}</strong>
                  <small>
                    {details?.developer_name ??
                      app.developer_name ??
                      "Unknown developer"}
                  </small>
                </span>
                <button
                  type="button"
                  aria-label="Close app details"
                  onClick={() => closeCard(true)}
                >
                  <X size={13} />
                </button>
              </header>
              <dl>
                <div>
                  <dt>Rank</dt>
                  <dd>#{app.position}</dd>
                </div>
                <div>
                  <dt>Store</dt>
                  <dd>
                    {countryFlag(country)} {country.toUpperCase()}
                  </dd>
                </div>
                <div>
                  <dt>Rating</dt>
                  <dd>
                    {details?.rating != null || app.rating != null ? (
                      <>
                        <Star size={10} fill="currentColor" />
                        {(details?.rating ?? app.rating)?.toFixed(1)}
                      </>
                    ) : (
                      "—"
                    )}
                  </dd>
                </div>
                <div>
                  <dt>Ratings</dt>
                  <dd>
                    {formatCount(details?.rating_count ?? app.rating_count)}
                  </dd>
                </div>
                <div>
                  <dt>Version</dt>
                  <dd>{details?.version ?? "—"}</dd>
                </div>
                <div>
                  <dt>Price</dt>
                  <dd>{details?.display_price ?? "—"}</dd>
                </div>
                <div className="wide">
                  <dt>Category</dt>
                  <dd>{details?.primary_category ?? "—"}</dd>
                </div>
                <div className="wide">
                  <dt>App Store ID</dt>
                  <dd>{app.apple_id}</dd>
                </div>
              </dl>
              {loading ? (
                <div className="ranking-card-status" role="status">
                  Loading current App Store numbers…
                </div>
              ) : error ? (
                <div className="ranking-card-status error" role="alert">
                  Current numbers unavailable. Cached ranking data is shown.
                </div>
              ) : null}
              <footer>
                <span>
                  “{keyword}” · {countryLabel(country)}
                </span>
                <a href={appStoreUrl} target="_blank" rel="noreferrer">
                  Open <ExternalLink size={11} />
                </a>
              </footer>
            </div>,
            document.body,
          )
        : null}
    </>
  );
}

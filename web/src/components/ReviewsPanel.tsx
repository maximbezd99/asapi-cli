import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Globe2, MessageSquare, Star } from "lucide-react";
import { api } from "../api";
import {
  countryFlag,
  countryLabel,
  formatCount,
  relativeTime,
} from "../format";
import type { Review, ReviewSummary, Storefront } from "../types";
import { useAsyncScope } from "../useAsyncScope";
import Picker from "./Picker";
import type { PickerOption } from "./Picker";

interface Props {
  projectId: string;
  appId: number;
  country: string;
  storefronts: Storefront[];
  summary?: ReviewSummary;
  onCountryChange: (country: string) => void;
  onTotalChange: (total: number) => void;
}

interface StorefrontReview extends Review {
  country: string;
}

export default function ReviewsPanel({
  projectId,
  appId,
  country,
  storefronts,
  summary,
  onCountryChange,
  onTotalChange,
}: Props) {
  const [reviews, setReviews] = useState<StorefrontReview[]>([]);
  const [page, setPage] = useState(0);
  const [hasMore, setHasMore] = useState(true);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [rating, setRating] = useState<number | undefined>();
  const [displayTotal, setDisplayTotal] = useState(summary?.count ?? 0);
  const [ratingCounts, setRatingCounts] = useState<number[]>(
    summary?.rating_counts ?? [0, 0, 0, 0, 0],
  );
  const [averageRating, setAverageRating] = useState<number | null>(
    summary?.average_rating ?? null,
  );
  const [lastUpdated, setLastUpdated] = useState<string | null>(
    summary?.page_one_updated_at ?? null,
  );
  const loadingRef = useRef(false);
  const sentinel = useRef<HTMLDivElement | null>(null);
  const reviewScope = `${projectId}:${appId}:${country}:${rating ?? "all"}:${
    summary?.page_one_updated_at ?? ""
  }:${summary?.count ?? 0}:${storefronts
    .map((storefront) => storefront.country)
    .join(",")}`;
  const {
    begin: beginReviewRequest,
    isCurrent: isCurrentReviewRequest,
  } = useAsyncScope(reviewScope);

  const pickerOptions = useMemo<PickerOption[]>(
    () => [
      {
        value: "all",
        label: "All storefronts",
        triggerLabel: "All",
        meta: `${storefronts.length} configured`,
        icon: <Globe2 size={13} />,
      },
      ...storefronts.map((storefront) => ({
        value: storefront.country,
        label: countryLabel(storefront.country),
        triggerLabel: storefront.country.toUpperCase(),
        meta: `${storefront.country.toUpperCase()}${
          storefront.is_main ? " · Main" : ""
        }`,
        icon: countryFlag(storefront.country),
      })),
    ],
    [storefronts],
  );

  const loadNext = useCallback(async () => {
    if (loadingRef.current || !hasMore || page >= 10 || error) return;
    const token = beginReviewRequest();
    loadingRef.current = true;
    setLoading(true);
    setError("");
    const nextPage = page + 1;
    const targetCountries =
      country === "all"
        ? storefronts.map((storefront) => storefront.country)
        : [country];
    try {
      const results = await Promise.all(
        targetCountries.map(async (targetCountry) => ({
          country: targetCountry,
          page: await api.reviews(
            projectId,
            appId,
            targetCountry,
            nextPage,
            rating,
          ),
        })),
      );
      if (!isCurrentReviewRequest(token)) return;
      if (results.some((result) => result.page.country !== result.country)) {
        throw new Error(
          "The server returned reviews for a different storefront. Retry this page.",
        );
      }
      setReviews((existing) => {
        const ids = new Set(
          existing.map((review) => `${review.country}:${review.review_id}`),
        );
        const incoming = results
          .flatMap(({ country: resultCountry, page: result }) =>
            result.reviews.map((review) => ({
              ...review,
              country: resultCountry,
            })),
          )
          .filter(
            (review) => !ids.has(`${review.country}:${review.review_id}`),
          );
        return [...existing, ...incoming].sort((left, right) =>
          (right.updated_at ?? "").localeCompare(left.updated_at ?? ""),
        );
      });
      const nextRatingCounts = results.reduce(
        (totals, result) =>
          totals.map(
            (count, index) => count + result.page.rating_counts[index],
          ),
        [0, 0, 0, 0, 0],
      );
      const ratingsTotal = nextRatingCounts.reduce(
        (total, count) => total + count,
        0,
      );
      const ratingsSum = nextRatingCounts.reduce(
        (total, count, index) => total + count * (index + 1),
        0,
      );
      setPage(nextPage);
      setHasMore(
        nextPage < 10 && results.some((result) => result.page.has_more),
      );
      setDisplayTotal(
        results.reduce((total, result) => total + result.page.total, 0),
      );
      setRatingCounts(nextRatingCounts);
      setAverageRating(ratingsTotal ? ratingsSum / ratingsTotal : null);
      setLastUpdated(
        results
          .map((result) => result.page.fetched_at)
          .filter((date): date is string => Boolean(date))
          .sort()
          .at(-1) ?? null,
      );
      onTotalChange(
        results.reduce((total, result) => total + result.page.total_all, 0),
      );
    } catch (reason) {
      if (isCurrentReviewRequest(token)) {
        setError((reason as Error).message);
      }
    } finally {
      if (isCurrentReviewRequest(token)) {
        loadingRef.current = false;
        setLoading(false);
      }
    }
  }, [
    beginReviewRequest,
    hasMore,
    page,
    projectId,
    appId,
    country,
    storefronts,
    rating,
    error,
    isCurrentReviewRequest,
    onTotalChange,
  ]);

  useEffect(() => {
    loadingRef.current = false;
    setReviews([]);
    setPage(0);
    setHasMore(true);
    setLoading(false);
    setError("");
    setDisplayTotal(summary?.count ?? 0);
    setRatingCounts(summary?.rating_counts ?? [0, 0, 0, 0, 0]);
    setAverageRating(summary?.average_rating ?? null);
    setLastUpdated(summary?.page_one_updated_at ?? null);
  }, [
    projectId,
    appId,
    country,
    rating,
    summary?.count,
    summary?.average_rating,
    summary?.page_one_updated_at,
    summary?.rating_counts,
  ]);

  useEffect(() => {
    if (page === 0 && hasMore && !loading) void loadNext();
  }, [page, hasMore, loading, loadNext]);

  useEffect(() => {
    const node = sentinel.current;
    if (!node) return;
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0]?.isIntersecting) void loadNext();
      },
      {
        root: null,
        rootMargin: "360px 0px",
      },
    );
    observer.observe(node);
    return () => observer.disconnect();
  }, [loadNext]);

  return (
    <section className="reviews-panel">
      <div className="reviews-heading">
        <h2>Reviews</h2>
        <div className="reviews-heading-actions">
          <Picker
            value={country}
            options={pickerOptions}
            onChange={onCountryChange}
            ariaLabel="Reviews storefront"
            className="tab-country-picker"
            searchPlaceholder="Search storefronts"
          />
          <span className="reviews-count">{formatCount(displayTotal)}</span>
        </div>
      </div>
      <div className="reviews-summary">
        <strong>{averageRating?.toFixed(1) ?? "—"}</strong>
        <span>
          <span className="stars">
            {Array.from({ length: 5 }, (_, index) => (
              <Star
                key={index}
                size={12}
                fill={
                  index < Math.round(averageRating ?? 0)
                    ? "currentColor"
                    : "none"
                }
              />
            ))}
          </span>
          <small>Updated {relativeTime(lastUpdated)}</small>
        </span>
        <div className="review-filters" aria-label="Filter reviews by rating">
          <button
            type="button"
            className={rating == null ? "active" : ""}
            aria-pressed={rating == null}
            onClick={() => setRating(undefined)}
          >
            All{" "}
            <small>
              {formatCount(ratingCounts.reduce((a, b) => a + b, 0))}
            </small>
          </button>
          {[5, 4, 3, 2, 1].map((stars) => (
            <button
              type="button"
              className={rating === stars ? "active" : ""}
              aria-pressed={rating === stars}
              onClick={() => setRating(stars)}
              key={stars}
            >
              <Star size={10} fill="currentColor" />
              {stars}
              <small>{formatCount(ratingCounts[stars - 1])}</small>
            </button>
          ))}
        </div>
      </div>
      <div className="review-list">
        {reviews.map((review) => (
          <article key={`${review.country}:${review.review_id}`}>
            <div className="review-title">
              <strong>{review.title || "Untitled review"}</strong>
              <span>
                <Star size={11} fill="currentColor" />
                {review.rating}
              </span>
            </div>
            <p
              className="review-content"
              tabIndex={0}
              aria-label={`Review text for ${review.title || "untitled review"}`}
            >
              {review.content}
            </p>
            <footer>
              <span>{review.author || "Anonymous"}</span>
              <span>
                {countryFlag(review.country)} {review.country.toUpperCase()}
                {" · "}
                {review.version ? `v${review.version} · ` : ""}
                {relativeTime(review.updated_at)}
              </span>
            </footer>
          </article>
        ))}
        {!loading && !reviews.length && !error ? (
          <div className="no-reviews">
            <MessageSquare size={20} />
            {country === "all"
              ? "No reviews in the configured storefronts."
              : "No reviews in this storefront."}
          </div>
        ) : null}
        {error ? (
          <div className="review-error" role="alert">
            <span>{error}</span>
            <button type="button" onClick={() => setError("")}>
              Retry
            </button>
          </div>
        ) : null}
        {loading ? (
          <div className="review-loading" role="status">
            <span className="registry-loader" aria-hidden="true">
              <i />
              <i />
              <i />
            </span>
            <span>
              Loading {country === "all" ? "storefronts" : "next page"}
            </span>
          </div>
        ) : null}
        <div ref={sentinel} className="review-sentinel" />
      </div>
    </section>
  );
}

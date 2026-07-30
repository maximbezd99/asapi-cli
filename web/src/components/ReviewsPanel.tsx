import { useCallback, useEffect, useRef, useState } from "react";
import { MessageSquare, Star } from "lucide-react";
import { api } from "../api";
import { formatCount, relativeTime } from "../format";
import type { Review, ReviewSummary } from "../types";

interface Props {
  projectId: string;
  appId: number;
  country: string;
  summary?: ReviewSummary;
  onTotalChange: (total: number) => void;
}

export default function ReviewsPanel({
  projectId,
  appId,
  country,
  summary,
  onTotalChange,
}: Props) {
  const [reviews, setReviews] = useState<Review[]>([]);
  const [page, setPage] = useState(0);
  const [hasMore, setHasMore] = useState(true);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [rating, setRating] = useState<number | undefined>();
  const [displayTotal, setDisplayTotal] = useState(summary?.count ?? 0);
  const [ratingCounts, setRatingCounts] = useState<number[]>(
    summary?.rating_counts ?? [0, 0, 0, 0, 0],
  );
  const [viewportPagination, setViewportPagination] = useState(false);
  const reviewList = useRef<HTMLDivElement | null>(null);
  const sentinel = useRef<HTMLDivElement | null>(null);

  const loadNext = useCallback(async () => {
    if (loading || !hasMore || page >= 10 || error) return;
    setLoading(true);
    setError("");
    const nextPage = page + 1;
    try {
      const result = await api.reviews(
        projectId,
        appId,
        country,
        nextPage,
        rating,
      );
      setReviews((existing) => {
        const ids = new Set(existing.map((review) => review.review_id));
        return [
          ...existing,
          ...result.reviews.filter((review) => !ids.has(review.review_id)),
        ];
      });
      setPage(nextPage);
      setHasMore(nextPage < 10 && result.has_more);
      setDisplayTotal(result.total);
      setRatingCounts(result.rating_counts);
      onTotalChange(result.total_all);
    } catch (reason) {
      setError((reason as Error).message);
    } finally {
      setLoading(false);
    }
  }, [
    loading,
    hasMore,
    page,
    projectId,
    appId,
    country,
    rating,
    error,
    onTotalChange,
  ]);

  useEffect(() => {
    setReviews([]);
    setPage(0);
    setHasMore(true);
    setError("");
  }, [projectId, appId, country, rating]);

  useEffect(() => {
    setDisplayTotal(summary?.count ?? 0);
    setRatingCounts(summary?.rating_counts ?? [0, 0, 0, 0, 0]);
  }, [summary?.count, summary?.rating_counts]);

  useEffect(() => {
    if (page === 0 && hasMore && !loading) void loadNext();
  }, [page, hasMore, loading, loadNext]);

  useEffect(() => {
    const media = window.matchMedia("(max-width: 760px)");
    const updatePaginationRoot = () => setViewportPagination(media.matches);
    updatePaginationRoot();
    media.addEventListener("change", updatePaginationRoot);
    return () => media.removeEventListener("change", updatePaginationRoot);
  }, []);

  useEffect(() => {
    const node = sentinel.current;
    const root = viewportPagination ? null : reviewList.current;
    if (!node || (!viewportPagination && !root)) return;
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0]?.isIntersecting) void loadNext();
      },
      {
        root,
        rootMargin: viewportPagination ? "360px 0px" : "180px",
      },
    );
    observer.observe(node);
    return () => observer.disconnect();
  }, [loadNext, viewportPagination]);

  return (
    <section className="reviews-panel">
      <div className="reviews-heading">
        <h2>Reviews</h2>
        <span>{formatCount(displayTotal)}</span>
      </div>
      <div className="reviews-summary">
        <strong>{summary?.average_rating?.toFixed(1) ?? "—"}</strong>
        <span>
          <span className="stars">
            {Array.from({ length: 5 }, (_, index) => (
              <Star
                key={index}
                size={12}
                fill={
                  index < Math.round(summary?.average_rating ?? 0)
                    ? "currentColor"
                    : "none"
                }
              />
            ))}
          </span>
          <small>
            Updated {relativeTime(summary?.page_one_updated_at ?? null)}
          </small>
        </span>
        <div className="review-filters" aria-label="Filter reviews by rating">
          <button
            type="button"
            className={rating == null ? "active" : ""}
            aria-pressed={rating == null}
            onClick={() => setRating(undefined)}
          >
            All <small>{formatCount(ratingCounts.reduce((a, b) => a + b, 0))}</small>
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
      <div ref={reviewList} className="review-list">
        {reviews.map((review) => (
          <article key={review.review_id}>
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
                {review.version ? `v${review.version} · ` : ""}
                {relativeTime(review.updated_at)}
              </span>
            </footer>
          </article>
        ))}
        {!loading && !reviews.length && !error ? (
          <div className="no-reviews">
            <MessageSquare size={20} />
            No reviews in this storefront.
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
            <span>Loading next page</span>
          </div>
        ) : null}
        <div ref={sentinel} className="review-sentinel" />
      </div>
    </section>
  );
}

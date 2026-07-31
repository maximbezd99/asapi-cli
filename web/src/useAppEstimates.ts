import { useCallback, useEffect, useMemo, useState } from "react";
import { api } from "./api";
import type { MarketEstimate } from "./types";

export function useAppEstimates(appIds: number[]) {
  const stableIds = useMemo(
    () => [...new Set(appIds)].sort((left, right) => left - right),
    [appIds.join(",")],
  );
  const [estimates, setEstimates] = useState(
    () => new Map<number, MarketEstimate>(),
  );
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [requestVersion, setRequestVersion] = useState(0);

  const retry = useCallback(() => {
    setRequestVersion((version) => version + 1);
  }, []);

  useEffect(() => {
    let cancelled = false;
    setEstimates(new Map());
    setError("");
    if (!stableIds.length) {
      setLoading(false);
      return () => {
        cancelled = true;
      };
    }
    setLoading(true);
    api
      .appEstimates(stableIds)
      .then((items) => {
        if (!cancelled) {
          setEstimates(new Map(items.map((item) => [item.app_id, item])));
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
  }, [requestVersion, stableIds]);

  return { estimates, loading, error, retry };
}

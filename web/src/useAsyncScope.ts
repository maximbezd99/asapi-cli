import { useCallback, useEffect, useRef } from "react";

export interface AsyncScopeToken {
  scope: string;
  sequence: number;
}

export function useAsyncScope(scope: string) {
  const state = useRef({ scope, sequence: 0, mounted: true });

  if (state.current.scope !== scope) {
    state.current.scope = scope;
    state.current.sequence += 1;
  }

  useEffect(() => {
    state.current.mounted = true;
    return () => {
      state.current.mounted = false;
      state.current.sequence += 1;
    };
  }, []);

  const begin = useCallback((): AsyncScopeToken => {
    state.current.sequence += 1;
    return {
      scope: state.current.scope,
      sequence: state.current.sequence,
    };
  }, []);

  const isCurrent = useCallback(
    (token: AsyncScopeToken) =>
      state.current.mounted &&
      state.current.scope === token.scope &&
      state.current.sequence === token.sequence,
    [],
  );

  return { begin, isCurrent };
}

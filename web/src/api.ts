import type {
  AppDetails,
  AppSummary,
  AppView,
  Country,
  Envelope,
  Keyword,
  Project,
  ReviewsPage,
  Storefront,
} from "./types";

interface ResponseEnvelope<T> {
  data: T;
}

export class ApiError extends Error {
  constructor(
    message: string,
    readonly status: number,
  ) {
    super(message);
  }
}

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const response = await fetch(path, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      ...options?.headers,
    },
  });
  if (!response.ok) {
    const body = (await response.json().catch(() => null)) as
      | { error?: { message?: string } }
      | null;
    throw new ApiError(
      body?.error?.message ?? `Request failed with HTTP ${response.status}`,
      response.status,
    );
  }
  if (response.status === 204) {
    return undefined as T;
  }
  return response.json() as Promise<T>;
}

export const api = {
  projects: async () =>
    (await request<ResponseEnvelope<Project[]>>("/api/v1/projects")).data,

  createProject: async (name: string) =>
    (
      await request<ResponseEnvelope<Project>>("/api/v1/projects", {
        method: "POST",
        body: JSON.stringify({ name }),
      })
    ).data,

  countries: async () =>
    await request<Country[]>("/api/v1/query/list/countries"),

  apps: async (projectId: string) =>
    (
      await request<ResponseEnvelope<AppSummary[]>>(
        `/api/v1/projects/${projectId}/apps`,
      )
    ).data,

  addApp: async (projectId: string, appId: number, country = "us") =>
    (
      await request<ResponseEnvelope<AppView>>(
        `/api/v1/projects/${projectId}/apps`,
        {
          method: "POST",
          body: JSON.stringify({ app_id: appId, country }),
        },
      )
    ).data,

  app: async (projectId: string, appId: number, country?: string) =>
    (
      await request<ResponseEnvelope<AppView>>(
        `/api/v1/projects/${projectId}/apps/${appId}${
          country ? `?country=${encodeURIComponent(country)}` : ""
        }`,
      )
    ).data,

  refreshApp: async (
    projectId: string,
    appId: number,
    options: { country?: string; all?: boolean } = {},
  ) =>
    (
      await request<ResponseEnvelope<AppView>>(
        `/api/v1/projects/${projectId}/apps/${appId}/refresh`,
        {
          method: "POST",
          body: JSON.stringify(options),
        },
      )
    ).data,

  deleteApp: async (projectId: string, appId: number) =>
    await request<void>(`/api/v1/projects/${projectId}/apps/${appId}`, {
      method: "DELETE",
    }),

  lookupApp: async (appId: number, country: string) =>
    (
      await request<Envelope<AppDetails[]>>("/api/v1/query/lookup", {
        method: "POST",
        body: JSON.stringify({
          apps: [{ id: appId }],
          country,
          full: false,
        }),
      })
    ).data[0] ?? null,

  reviews: async (
    projectId: string,
    appId: number,
    country: string,
    page: number,
    rating?: number,
  ) =>
    (
      await request<ResponseEnvelope<ReviewsPage>>(
        `/api/v1/projects/${projectId}/apps/${appId}/reviews?country=${encodeURIComponent(
          country,
        )}&page=${page}${rating ? `&rating=${rating}` : ""}`,
      )
    ).data,

  keywords: async (projectId: string, appId: number) =>
    (
      await request<ResponseEnvelope<Keyword[]>>(
        `/api/v1/projects/${projectId}/apps/${appId}/keywords`,
      )
    ).data,

  refreshKeywords: async (
    projectId: string,
    appId: number,
    queryId: number,
    force = false,
  ) =>
    (
      await request<ResponseEnvelope<Keyword[]>>(
        `/api/v1/projects/${projectId}/apps/${appId}/keywords/refresh`,
        {
          method: "POST",
          body: JSON.stringify({ query_id: queryId, force }),
        },
      )
    ).data,

  addKeyword: async (
    projectId: string,
    appId: number,
    keyword: string,
    country: string,
    notes = "",
  ) =>
    (
      await request<ResponseEnvelope<Keyword>>(
        `/api/v1/projects/${projectId}/apps/${appId}/keywords`,
        {
          method: "POST",
          body: JSON.stringify({ keyword, country, notes }),
        },
      )
    ).data,

  deleteKeyword: async (projectId: string, appId: number, queryId: number) =>
    await request<void>(
      `/api/v1/projects/${projectId}/apps/${appId}/keywords/${queryId}`,
      { method: "DELETE" },
    ),

  addStorefront: async (
    projectId: string,
    appId: number,
    country: string,
    autoRefresh: boolean,
  ) =>
    (
      await request<ResponseEnvelope<Storefront>>(
        `/api/v1/projects/${projectId}/apps/${appId}/storefronts`,
        {
          method: "POST",
          body: JSON.stringify({
            country,
            auto_refresh: autoRefresh,
          }),
        },
      )
    ).data,

  updateStorefront: async (
    projectId: string,
    appId: number,
    country: string,
    changes: { is_main?: boolean; auto_refresh?: boolean },
  ) =>
    (
      await request<ResponseEnvelope<Storefront[]>>(
        `/api/v1/projects/${projectId}/apps/${appId}/storefronts/${country}`,
        {
          method: "PATCH",
          body: JSON.stringify(changes),
        },
      )
    ).data,

  deleteStorefront: async (
    projectId: string,
    appId: number,
    country: string,
  ) =>
    await request<void>(
      `/api/v1/projects/${projectId}/apps/${appId}/storefronts/${country}`,
      { method: "DELETE" },
    ),
};

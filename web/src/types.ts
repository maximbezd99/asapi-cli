export interface Project {
  id: string;
  name: string;
  created_at: string;
}

export interface Storefront {
  country: string;
  is_main: boolean;
  auto_refresh: boolean;
  created_at: string;
}

export interface Country {
  code: string;
  name: string;
}

export interface AppSummary {
  apple_id: number;
  created_at: string;
  main_country: string;
  name: string | null;
  icon_url: string | null;
  rating: number | null;
  rating_count: number | null;
  version: string | null;
  last_updated: string | null;
}

export interface Envelope<T> {
  data: T;
  meta: {
    country: string | null;
    retrieved_at: string;
    result_count: number;
  };
}

export interface AppDetails {
  app_id: number;
  name: string;
  developer_name?: string;
  developer_id?: number;
  primary_category?: string;
  display_price?: string;
  rating?: number;
  rating_count?: number;
  version?: string;
  minimum_os_version?: string;
  content_rating?: string;
  description?: string;
  release_notes?: string;
  categories?: string[];
  seller_name?: string;
  developer_website_url?: string;
  app_store_url?: string;
  icon_url?: string;
  screenshots?: string[];
  languages?: string[];
  size_bytes?: number;
  released_at?: string;
  version_released_at?: string;
  humanized_worldwide_last_month_downloads?: HumanizedDownloads;
  humanized_worldwide_last_month_revenue?: HumanizedRevenue;
  has_in_app_purchases?: boolean;
  has_external_purchases?: boolean;
  in_app_purchases?: Purchase[];
  similar_apps?: SimilarApp[];
}

export interface SearchApp {
  position: number;
  app_id: number;
  name: string;
  developer_name?: string;
  primary_category?: string;
  display_price?: string;
  rating?: number;
  rating_count?: number;
  version?: string;
  app_store_url?: string;
  icon_url?: string;
  released_at?: string;
  version_released_at?: string;
}

export interface HumanizedDownloads {
  downloads: number;
  downloads_rounded: number;
  prefix?: string;
  string: string;
  units: string;
}

export interface HumanizedRevenue {
  prefix?: string;
  revenue: number;
  revenue_rounded: number;
  string: string;
  units: string;
}

export interface MarketEstimate {
  app_id: number;
  humanized_worldwide_last_month_downloads?: HumanizedDownloads;
  humanized_worldwide_last_month_revenue?: HumanizedRevenue;
}

export interface AppEstimateMetric {
  value: number;
  rounded_value: number;
  prefix: string | null;
  display: string;
  units: string;
}

export interface AppEstimates {
  fetched_at: string;
  source: string;
  scope: "worldwide";
  period: "last_month";
  downloads: AppEstimateMetric | null;
  revenue: AppEstimateMetric | null;
  revenue_currency: string | null;
}

export interface Purchase {
  name: string;
  display_price: string;
}

export interface SimilarApp {
  app_id: number;
  name: string;
  developer_name?: string;
  icon_url?: string;
  rating?: number;
}

export interface PopularityCountry {
  country: string;
  available: boolean;
  name: string | null;
  rating: number | null;
  rating_count: number | null;
}

export interface Popularity {
  fetched_at: string;
  group: string | null;
  countries: PopularityCountry[];
}

export interface ReviewSummary {
  count: number;
  average_rating: number | null;
  page_one_updated_at: string | null;
  rating_counts: number[];
}

export interface AppView {
  apple_id: number;
  created_at: string;
  selected_country: string;
  storefronts: Storefront[];
  details: Envelope<AppDetails[]> | null;
  details_updated_at: string | null;
  estimates: AppEstimates | null;
  iap: Envelope<{
    has_in_app_purchases: boolean;
    has_external_purchases: boolean;
    purchases: { name: string; display_price: string }[];
  }> | null;
  similar: Envelope<
    {
      app_id: number;
      name: string;
      developer_name?: string;
      icon_url?: string;
      rating?: number;
    }[]
  > | null;
  popularity: Popularity | null;
  review_summary: ReviewSummary;
}

export interface Review {
  review_id: number;
  author: string | null;
  rating: number;
  title: string | null;
  content: string;
  version: string | null;
  updated_at: string | null;
}

export interface ReviewsPage {
  country: string;
  page: number;
  page_size: number;
  total: number;
  total_all: number;
  has_more: boolean;
  rating_counts: number[];
  fetched_at: string | null;
  reviews: Review[];
}

export interface RankedApp {
  position: number;
  apple_id: number;
  name: string;
  icon_url: string | null;
  developer_name: string | null;
  released_at: string | null;
  version_released_at: string | null;
  rating: number | null;
  rating_count: number | null;
}

export interface TrendPoint {
  fetched_at: string;
  position: number | null;
}

export interface KeywordEntity {
  query_id: number;
  keyword: string;
  normalized_keyword: string;
  country: string;
  notes: string;
  difficulty: number | null;
  popularity: number | null;
}

export interface Keyword extends KeywordEntity {
  last_updated: string | null;
  position: number | null;
  previous_position: number | null;
  trend: TrendPoint[];
  apps_in_ranking: RankedApp[];
}

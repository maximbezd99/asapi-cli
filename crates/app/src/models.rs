use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Storefront {
    pub country: String,
    pub is_main: bool,
    pub auto_refresh: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrackedAppSummary {
    pub apple_id: i64,
    pub created_at: String,
    pub main_country: String,
    pub name: Option<String>,
    pub icon_url: Option<String>,
    pub rating: Option<f64>,
    pub rating_count: Option<i64>,
    pub version: Option<String>,
    pub last_updated: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppView {
    pub apple_id: i64,
    pub created_at: String,
    pub selected_country: String,
    pub storefronts: Vec<Storefront>,
    pub details: Option<Value>,
    pub details_updated_at: Option<String>,
    pub estimates: Option<AppEstimatesView>,
    pub iap: Option<Value>,
    pub similar: Option<Value>,
    pub popularity: Option<PopularityView>,
    pub review_summary: ReviewSummary,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppEstimatesView {
    pub fetched_at: String,
    pub source: String,
    pub scope: String,
    pub period: String,
    pub downloads: Option<AppEstimateMetric>,
    pub revenue: Option<AppEstimateMetric>,
    pub revenue_currency: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppEstimateMetric {
    pub value: i64,
    pub rounded_value: i64,
    pub prefix: Option<String>,
    pub display: String,
    pub units: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HistorySnapshot {
    pub resource: String,
    pub country: Option<String>,
    pub fetched_at: String,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct PopularityView {
    pub fetched_at: String,
    pub group: Option<String>,
    pub countries: Vec<PopularityCountry>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct PopularityCountry {
    pub country: String,
    pub available: bool,
    pub name: Option<String>,
    pub rating: Option<f64>,
    pub rating_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ReviewSummary {
    pub count: i64,
    pub average_rating: Option<f64>,
    pub page_one_updated_at: Option<String>,
    pub rating_counts: [i64; 5],
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Review {
    pub review_id: i64,
    pub author: Option<String>,
    pub rating: i64,
    pub title: Option<String>,
    pub content: String,
    pub version: Option<String>,
    pub helpful_score: Option<i64>,
    pub helpful_vote_count: Option<i64>,
    pub updated_at: Option<String>,
    pub first_seen_at: String,
    pub last_seen_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReviewsPage {
    pub country: String,
    pub page: u8,
    pub page_size: u32,
    pub total: i64,
    pub total_all: i64,
    pub has_more: bool,
    pub rating_counts: [i64; 5],
    pub fetched_at: Option<String>,
    pub reviews: Vec<Review>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KeywordEntity {
    pub query_id: i64,
    pub keyword: String,
    pub normalized_keyword: String,
    pub country: String,
    pub notes: String,
    pub difficulty: Option<f64>,
    pub popularity: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KeywordView {
    #[serde(flatten)]
    pub entity: KeywordEntity,
    pub last_updated: Option<String>,
    pub position: Option<i64>,
    pub previous_position: Option<i64>,
    pub trend: Vec<KeywordTrendPoint>,
    pub apps_in_ranking: Vec<RankedApp>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct KeywordTrendPoint {
    pub fetched_at: String,
    pub position: Option<i64>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct RankedApp {
    pub position: i64,
    pub apple_id: i64,
    pub name: String,
    pub icon_url: Option<String>,
    pub developer_name: Option<String>,
    pub released_at: Option<String>,
    pub version_released_at: Option<String>,
    pub rating: Option<f64>,
    pub rating_count: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProject {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct RenameProject {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct AddApp {
    pub app_id: u64,
    #[serde(default = "default_country")]
    pub country: String,
}

#[derive(Debug, Deserialize)]
pub struct AddStorefront {
    pub country: String,
    #[serde(default)]
    pub auto_refresh: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStorefront {
    pub is_main: Option<bool>,
    pub auto_refresh: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct AddKeyword {
    pub keyword: String,
    pub country: String,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateKeyword {
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum PatchValue<T> {
    #[default]
    Missing,
    Present(Option<T>),
}

impl<T> PatchValue<T> {
    pub fn is_present(&self) -> bool {
        matches!(self, Self::Present(_))
    }

    pub fn apply(self, current: Option<T>) -> Option<T> {
        match self {
            Self::Missing => current,
            Self::Present(value) => value,
        }
    }
}

impl<'de, T> Deserialize<'de> for PatchValue<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<T>::deserialize(deserializer).map(Self::Present)
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateKeywordMetrics {
    pub keyword: String,
    pub country: String,
    #[serde(default)]
    pub difficulty: PatchValue<f64>,
    #[serde(default)]
    pub popularity: PatchValue<f64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct RefreshApp {
    pub country: Option<String>,
    #[serde(default)]
    pub all: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct RefreshKeyword {
    pub query_id: Option<i64>,
    #[serde(default)]
    pub force: bool,
}

fn default_country() -> String {
    "us".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyword_metric_patch_distinguishes_missing_null_and_value() {
        let missing: UpdateKeywordMetrics =
            serde_json::from_str(r#"{"keyword":"music","country":"us"}"#).unwrap();
        assert_eq!(missing.difficulty, PatchValue::Missing);

        let null: UpdateKeywordMetrics =
            serde_json::from_str(r#"{"keyword":"music","country":"us","difficulty":null}"#)
                .unwrap();
        assert_eq!(null.difficulty, PatchValue::Present(None));

        let value: UpdateKeywordMetrics =
            serde_json::from_str(r#"{"keyword":"music","country":"us","popularity":72.5}"#)
                .unwrap();
        assert_eq!(value.popularity, PatchValue::Present(Some(72.5)));
    }
}

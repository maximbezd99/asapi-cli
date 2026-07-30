use serde::{Deserialize, Serialize};

use crate::app_store::AppSpecifier;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub term: String,
    pub country: Option<String>,
    #[serde(default = "default_search_limit")]
    pub limit: u32,
    pub local_limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookupRequest {
    pub apps: Vec<AppSpecifier>,
    pub country: Option<String>,
    #[serde(default)]
    pub full: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PopularityGroup {
    #[default]
    Tier1,
    Tier2,
}

impl PopularityGroup {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tier1 => "tier1",
            Self::Tier2 => "tier2",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopularityRequest {
    pub app: AppSpecifier,
    #[serde(default)]
    pub group: PopularityGroup,
    pub countries: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IapRequest {
    pub app: AppSpecifier,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarRequest {
    pub app: AppSpecifier,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewsRequest {
    pub app: AppSpecifier,
    pub country: Option<String>,
    pub page: Option<u8>,
    pub pages: Option<u8>,
    #[serde(default)]
    pub all: bool,
}

impl ReviewsRequest {
    pub fn requested_pages(&self) -> Vec<u8> {
        if let Some(page) = self.page {
            vec![page]
        } else {
            (1..=self.pages.unwrap_or(if self.all { 10 } else { 1 })).collect()
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChartType {
    Top,
    Free,
    Paid,
    Grossing,
}

impl ChartType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Free => "free",
            Self::Paid => "paid",
            Self::Grossing => "grossing",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartRequest {
    pub chart: ChartType,
    pub country: Option<String>,
    #[serde(default = "default_chart_limit")]
    pub limit: u16,
    pub category: Option<u32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ListResource {
    Countries,
    Categories,
    ChartTypes,
}

fn default_search_limit() -> u32 {
    10
}

fn default_chart_limit() -> u16 {
    10
}

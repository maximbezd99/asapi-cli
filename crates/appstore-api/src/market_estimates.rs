use anyhow::{bail, Context, Result};
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::client::ApiClient;

const MAX_APP_IDS: usize = 100;
const ENDPOINT_BYTES: &[u8] = &[
    104, 116, 116, 112, 115, 58, 47, 47, 97, 112, 112, 46, 115, 101, 110, 115, 111, 114, 116, 111,
    119, 101, 114, 46, 99, 111, 109, 47, 97, 112, 105, 47, 105, 111, 115, 47, 97, 112, 112, 115,
];

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct HumanizedDownloads {
    pub downloads: u64,
    pub downloads_rounded: u64,
    pub prefix: Option<String>,
    pub string: String,
    pub units: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct HumanizedRevenue {
    pub prefix: Option<String>,
    pub revenue: u64,
    pub revenue_rounded: u64,
    pub string: String,
    pub units: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct AppEstimates {
    pub app_id: u64,
    pub humanized_worldwide_last_month_downloads: Option<HumanizedDownloads>,
    pub humanized_worldwide_last_month_revenue: Option<HumanizedRevenue>,
}

#[derive(Debug, Deserialize)]
struct AppsResponse {
    apps: Vec<AppEstimates>,
}

pub async fn fetch_app_estimates(client: &ApiClient, app_ids: &[u64]) -> Result<Vec<AppEstimates>> {
    if app_ids.is_empty() {
        bail!("market estimates require at least one app ID");
    }

    let mut estimates = Vec::with_capacity(app_ids.len());
    for chunk in app_ids.chunks(MAX_APP_IDS) {
        let json = client.fetch_json(apps_url(chunk)?).await?;
        let mut response: AppsResponse =
            serde_json::from_value(json).context("invalid market estimates response")?;
        estimates.append(&mut response.apps);
    }
    Ok(estimates)
}

fn endpoint_url() -> Result<Url> {
    let endpoint =
        String::from_utf8(ENDPOINT_BYTES.to_vec()).context("invalid market estimates endpoint")?;
    Url::parse(&endpoint).context("invalid market estimates endpoint URL")
}

fn apps_url(app_ids: &[u64]) -> Result<Url> {
    if app_ids.is_empty() {
        bail!("market estimates require at least one app ID");
    }
    if app_ids.len() > MAX_APP_IDS {
        bail!("market estimates accept at most {MAX_APP_IDS} app IDs per request");
    }
    let ids = app_ids
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let mut url = endpoint_url()?;
    url.query_pairs_mut().append_pair("app_ids", &ids);
    Ok(url)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn preserves_less_than_bucket_semantics() {
        let response: AppsResponse = serde_json::from_value(json!({
            "apps": [{
                "app_id": 1,
                "humanized_worldwide_last_month_downloads": {
                    "downloads": 1000,
                    "downloads_rounded": 5,
                    "prefix": "< ",
                    "string": "< 5k",
                    "units": "k"
                },
                "humanized_worldwide_last_month_revenue": {
                    "prefix": "< $",
                    "revenue": 1000,
                    "revenue_rounded": 5,
                    "string": "< $5k",
                    "units": "k"
                }
            }]
        }))
        .unwrap();

        let revenue = response.apps[0]
            .humanized_worldwide_last_month_revenue
            .as_ref()
            .unwrap();
        assert_eq!(revenue.string, "< $5k");
        assert_eq!(revenue.revenue, 1000);
    }
}

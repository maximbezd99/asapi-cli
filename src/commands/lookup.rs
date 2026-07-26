use anyhow::{bail, Result};
use chrono::{DateTime, FixedOffset};
use reqwest::Url;
use serde::Serialize;
use serde_json::{json, Value};

use super::{date, envelope, parse_result_array, strings, text, u32_field, ParseBatch};
use crate::{app_store::resolve_country, cli::LookupArgs, client::ApiClient, output::Envelope};

#[derive(Debug, Serialize, PartialEq)]
pub struct LookupApp {
    pub app_id: u64,
    pub bundle_id: Option<String>,
    pub name: String,
    pub developer_name: Option<String>,
    pub developer_id: Option<u64>,
    pub primary_category_id: Option<u32>,
    pub primary_category: Option<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub display_price: Option<String>,
    pub rating: Option<f64>,
    pub rating_count: Option<u64>,
    pub version: Option<String>,
    pub minimum_os_version: Option<String>,
    pub content_rating: Option<String>,
    pub description: Option<String>,
    pub release_notes: Option<String>,
    pub categories: Vec<String>,
    pub category_ids: Vec<String>,
    pub seller_name: Option<String>,
    pub app_store_url: Option<String>,
    pub icon_url: Option<String>,
    pub screenshots: Vec<String>,
    pub languages: Vec<String>,
    pub size_bytes: Option<u64>,
    pub released_at: Option<DateTime<FixedOffset>>,
    pub version_released_at: Option<DateTime<FixedOffset>>,
}

pub async fn run(client: &ApiClient, args: &LookupArgs) -> Result<Envelope> {
    if args.apps.len() > 10 {
        bail!("lookup accepts at most 10 apps per request");
    }
    let ids = args.apps.iter().map(|app| app.id).collect::<Vec<_>>();
    let country = resolve_country(args.country.country.as_deref(), &args.apps)?;
    let json = client.fetch_json(lookup_url(&ids, &country)?).await?;
    let batch = parse(&json);
    envelope(
        "lookup",
        "Apple App Store",
        Some(country),
        json!({"app_ids": ids}),
        &batch.records,
        batch.skipped_count,
        None,
        None,
    )
}

pub(super) fn lookup_url(ids: &[u64], country: &str) -> Result<Url> {
    if ids.is_empty() {
        bail!("lookup requires at least one app ID");
    }
    let joined = ids.iter().map(u64::to_string).collect::<Vec<_>>().join(",");
    let mut url = Url::parse("https://itunes.apple.com/lookup")?;
    url.query_pairs_mut()
        .append_pair("id", &joined)
        .append_pair("country", country);
    Ok(url)
}

pub(super) fn parse(json: &Value) -> ParseBatch<LookupApp> {
    parse_result_array(json, |value, _| {
        Some(LookupApp {
            app_id: value.get("trackId")?.as_u64()?,
            name: text(value, "trackName")?,
            bundle_id: text(value, "bundleId"),
            developer_name: text(value, "artistName"),
            developer_id: value.get("artistId").and_then(Value::as_u64),
            primary_category_id: u32_field(value, "primaryGenreId"),
            primary_category: text(value, "primaryGenreName"),
            price: value.get("price").and_then(Value::as_f64),
            currency: text(value, "currency"),
            display_price: text(value, "formattedPrice"),
            rating: value.get("averageUserRating").and_then(Value::as_f64),
            rating_count: value.get("userRatingCount").and_then(Value::as_u64),
            version: text(value, "version"),
            minimum_os_version: text(value, "minimumOsVersion"),
            content_rating: text(value, "trackContentRating")
                .or_else(|| text(value, "contentAdvisoryRating")),
            description: text(value, "description"),
            release_notes: text(value, "releaseNotes"),
            categories: strings(value, "genres"),
            category_ids: strings(value, "genreIds"),
            seller_name: text(value, "sellerName"),
            app_store_url: text(value, "trackViewUrl"),
            icon_url: text(value, "artworkUrl512").or_else(|| text(value, "artworkUrl100")),
            screenshots: strings(value, "screenshotUrls"),
            languages: strings(value, "languageCodesISO2A"),
            size_bytes: value
                .get("fileSizeBytes")
                .and_then(Value::as_str)
                .and_then(|number| number.parse().ok()),
            released_at: date(value, "releaseDate"),
            version_released_at: date(value, "currentVersionReleaseDate"),
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_joins_ids() {
        assert!(lookup_url(&[1, 2], "gb")
            .unwrap()
            .as_str()
            .contains("id=1%2C2"));
    }
}

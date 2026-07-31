use std::collections::HashMap;

use anyhow::{bail, Result};
use chrono::{DateTime, FixedOffset};
use reqwest::Url;
use serde::Serialize;
use serde_json::{json, Value};

use super::{
    date, envelope, iap, parse_result_array, similar, strings, text, u32_field, ParseBatch,
};
use crate::{
    app_store::{product_page_url, resolve_country, ProductPagePayload},
    client::ApiClient,
    market_estimates::{self, HumanizedDownloads, HumanizedRevenue},
    output::Envelope,
    requests::LookupRequest,
};

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
    pub developer_website_url: Option<String>,
    pub app_store_url: Option<String>,
    pub icon_url: Option<String>,
    pub screenshots: Vec<String>,
    pub languages: Vec<String>,
    pub size_bytes: Option<u64>,
    pub released_at: Option<DateTime<FixedOffset>>,
    pub version_released_at: Option<DateTime<FixedOffset>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub humanized_worldwide_last_month_downloads: Option<HumanizedDownloads>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub humanized_worldwide_last_month_revenue: Option<HumanizedRevenue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_in_app_purchases: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_external_purchases: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_app_purchases: Option<Vec<iap::InAppPurchase>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub similar_apps: Option<Vec<similar::SimilarApp>>,
}

pub async fn run(client: &ApiClient, args: &LookupRequest) -> Result<Envelope> {
    run_with_estimates(client, args, true).await
}

#[doc(hidden)]
pub async fn run_without_estimates(client: &ApiClient, args: &LookupRequest) -> Result<Envelope> {
    run_with_estimates(client, args, false).await
}

async fn run_with_estimates(
    client: &ApiClient,
    args: &LookupRequest,
    include_estimates: bool,
) -> Result<Envelope> {
    if args.apps.len() > 10 {
        bail!("lookup accepts at most 10 apps per request");
    }
    let ids = args.apps.iter().map(|app| app.id).collect::<Vec<_>>();
    let country = resolve_country(args.country.as_deref(), &args.apps)?;
    let json = client.fetch_json(lookup_url(&ids, &country)?).await?;
    let mut batch = parse(&json);
    if args.full {
        if include_estimates {
            let (product_pages, estimates) = tokio::join!(
                enrich_all_from_product_pages(client, &mut batch.records, &country),
                market_estimates::fetch_app_estimates(client, &ids),
            );
            product_pages?;
            merge_estimates(&mut batch.records, estimates?);
        } else {
            enrich_all_from_product_pages(client, &mut batch.records, &country).await?;
        }
    }
    let mut result = envelope(
        "lookup",
        if args.full {
            if include_estimates {
                "Apple App Store, product page, and market estimates"
            } else {
                "Apple App Store and product page"
            }
        } else {
            "Apple App Store"
        },
        Some(country),
        json!({"app_ids": ids, "full": args.full}),
        &batch.records,
        batch.skipped_count,
        None,
        None,
    )?;
    if args.full {
        result.meta.coverage_note = Some(if include_estimates {
            "Screenshots, in-app purchases, and similar apps reflect the public storefront product page. Worldwide last-month downloads and revenue are third-party rounded estimates; preserve the displayed string because '<' buckets are upper-bound ranges, not exact values."
                .to_string()
        } else {
            "Screenshots, in-app purchases, and similar apps reflect the public storefront product page and may change or be incomplete."
                .to_string()
        });
    }
    Ok(result)
}

async fn enrich_all_from_product_pages(
    client: &ApiClient,
    apps: &mut [LookupApp],
    country: &str,
) -> Result<()> {
    for app in apps {
        enrich_from_product_page(client, app, country).await?;
    }
    Ok(())
}

fn merge_estimates(apps: &mut [LookupApp], estimates: Vec<market_estimates::AppEstimates>) {
    let mut estimates = estimates
        .into_iter()
        .map(|estimate| (estimate.app_id, estimate))
        .collect::<HashMap<_, _>>();
    for app in apps {
        if let Some(estimate) = estimates.remove(&app.app_id) {
            app.humanized_worldwide_last_month_downloads =
                estimate.humanized_worldwide_last_month_downloads;
            app.humanized_worldwide_last_month_revenue =
                estimate.humanized_worldwide_last_month_revenue;
        }
    }
}

async fn enrich_from_product_page(
    client: &ApiClient,
    app: &mut LookupApp,
    country: &str,
) -> Result<()> {
    let html = client
        .fetch_text(product_page_url(app.app_id, country)?)
        .await?;
    let payload = ProductPagePayload::parse(&html)?;
    let data = payload.app_data(app.app_id)?;
    let purchases = iap::parse_data(data, app.app_id)?;
    let similar = similar::parse(data);

    let product_page_screenshots = screenshots(data);
    if !product_page_screenshots.is_empty() {
        app.screenshots = product_page_screenshots;
    }
    if app.developer_website_url.is_none() {
        app.developer_website_url = developer_website_url(data);
    }
    app.has_in_app_purchases = Some(purchases.has_in_app_purchases);
    app.has_external_purchases = Some(purchases.has_external_purchases);
    app.in_app_purchases = Some(purchases.purchases);
    app.similar_apps = Some(similar.records);
    Ok(())
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
            developer_website_url: text(value, "sellerUrl"),
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
            humanized_worldwide_last_month_downloads: None,
            humanized_worldwide_last_month_revenue: None,
            has_in_app_purchases: None,
            has_external_purchases: None,
            in_app_purchases: None,
            similar_apps: None,
        })
    })
}

fn developer_website_url(data: &Value) -> Option<String> {
    match data {
        Value::Array(values) => values.iter().find_map(developer_website_url),
        Value::Object(object) => {
            let is_developer_website = ["title", "label", "name", "text"]
                .iter()
                .filter_map(|key| object.get(*key).and_then(Value::as_str))
                .any(|label| label.to_ascii_lowercase().contains("developer website"));
            if is_developer_website {
                ["actionUrl", "url", "href", "websiteUrl"]
                    .iter()
                    .filter_map(|key| object.get(*key))
                    .find_map(external_http_url)
                    .or_else(|| object.values().find_map(external_http_url))
            } else {
                object.values().find_map(developer_website_url)
            }
        }
        _ => None,
    }
}

fn external_http_url(value: &Value) -> Option<String> {
    match value {
        Value::String(url)
            if (url.starts_with("https://") || url.starts_with("http://"))
                && !url.contains("apps.apple.com") =>
        {
            Some(url.clone())
        }
        Value::Array(values) => values.iter().find_map(external_http_url),
        Value::Object(object) => object.values().find_map(external_http_url),
        _ => None,
    }
}

fn screenshots(data: &Value) -> Vec<String> {
    [
        "product_media_phone_",
        "product_media_pad_",
        "product_media_mac_",
    ]
    .iter()
    .find_map(|shelf| {
        let screenshots = data
            .pointer(&format!("/shelfMapping/{shelf}/items"))
            .and_then(Value::as_array)?
            .iter()
            .filter_map(|item| screenshot_url(item.get("screenshot")?))
            .collect::<Vec<_>>();
        (!screenshots.is_empty()).then_some(screenshots)
    })
    .unwrap_or_default()
}

fn screenshot_url(artwork: &Value) -> Option<String> {
    let template = text(artwork, "template")?;
    let source_width = artwork.get("width").and_then(Value::as_u64)?;
    let source_height = artwork.get("height").and_then(Value::as_u64)?;
    if source_width == 0 || source_height == 0 {
        return None;
    }
    let width = source_width.min(720);
    let height = source_height.saturating_mul(width) / source_width;
    let url = template
        .replace("{w}", &width.to_string())
        .replace("{h}", &height.to_string())
        .replace("{c}", "bb")
        .replace("{f}", "jpg");
    (!url.contains(['{', '}'])).then_some(url)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn url_joins_ids() {
        assert!(lookup_url(&[1, 2], "gb")
            .unwrap()
            .as_str()
            .contains("id=1%2C2"));
    }

    #[test]
    fn parses_phone_screenshots_from_product_page() {
        let data = json!({
            "shelfMapping": {
                "product_media_phone_": {
                    "items": [{
                        "screenshot": {
                            "template": "https://example.com/{w}x{h}{c}.{f}",
                            "width": 1320,
                            "height": 2868
                        }
                    }]
                }
            }
        });
        assert_eq!(screenshots(&data), ["https://example.com/720x1564bb.jpg"]);
    }

    #[test]
    fn parses_developer_website_from_product_page() {
        let data = json!({
            "items": [{
                "title": "Developer Website",
                "action": {
                    "url": "https://example.com/product"
                }
            }]
        });
        assert_eq!(
            developer_website_url(&data).as_deref(),
            Some("https://example.com/product")
        );
    }
}

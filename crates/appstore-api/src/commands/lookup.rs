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
    pub app_store_url: Option<String>,
    pub icon_url: Option<String>,
    pub screenshots: Vec<String>,
    pub languages: Vec<String>,
    pub size_bytes: Option<u64>,
    pub released_at: Option<DateTime<FixedOffset>>,
    pub version_released_at: Option<DateTime<FixedOffset>>,
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
    if args.apps.len() > 10 {
        bail!("lookup accepts at most 10 apps per request");
    }
    let ids = args.apps.iter().map(|app| app.id).collect::<Vec<_>>();
    let country = resolve_country(args.country.as_deref(), &args.apps)?;
    let json = client.fetch_json(lookup_url(&ids, &country)?).await?;
    let mut batch = parse(&json);
    if args.full {
        for app in &mut batch.records {
            enrich_from_product_page(client, app, &country).await?;
        }
    }
    let mut result = envelope(
        "lookup",
        if args.full {
            "Apple App Store and product page"
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
        result.meta.coverage_note = Some(
            "Screenshots, in-app purchases, and similar apps reflect the public storefront product page and may change or be incomplete."
                .to_string(),
        );
    }
    Ok(result)
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
            has_in_app_purchases: None,
            has_external_purchases: None,
            in_app_purchases: None,
            similar_apps: None,
        })
    })
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
}

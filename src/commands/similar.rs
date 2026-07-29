use anyhow::Result;
use serde::Serialize;
use serde_json::{json, Value};

use super::{envelope, parse_entries, text, ParseBatch};
use crate::{
    app_store::{product_page_url, resolve_country, ProductPagePayload},
    cli::SimilarArgs,
    client::ApiClient,
    output::Envelope,
};

#[derive(Debug, Serialize, PartialEq)]
pub struct SimilarApp {
    pub position: usize,
    pub app_id: u64,
    pub bundle_id: Option<String>,
    pub name: String,
    pub developer_name: Option<String>,
    pub subtitle: Option<String>,
    pub rating: Option<f64>,
    pub rating_count_display: Option<String>,
    pub offer: Option<String>,
    pub is_free: Option<bool>,
    pub has_in_app_purchases: Option<bool>,
    pub app_store_url: Option<String>,
    pub icon_url: Option<String>,
}

pub async fn run(client: &ApiClient, args: &SimilarArgs) -> Result<Envelope> {
    let app_id = args.app.id;
    let country = resolve_country(
        args.country.country.as_deref(),
        std::slice::from_ref(&args.app),
    )?;
    let html = client
        .fetch_text(product_page_url(app_id, &country)?)
        .await?;
    let payload = ProductPagePayload::parse(&html)?;
    let batch = parse(payload.app_data(app_id)?);
    let mut result = envelope(
        "similar",
        "Apple App Store product page",
        Some(country),
        json!({"app_id": app_id}),
        &batch.records,
        batch.skipped_count,
        None,
        Some(
            "One-based order in Apple's country-specific “You Might Also Like” shelf; not a universal similarity rank.",
        ),
    )?;
    result.meta.coverage_note = Some(
        "The result contains only the apps displayed in the public shelf and may change or be incomplete. rating_count_display is Apple's compact storefront label, not an exact count."
            .to_string(),
    );
    Ok(result)
}

fn parse(data: &Value) -> ParseBatch<SimilarApp> {
    let values = data
        .pointer("/shelfMapping/similarItems/items")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    parse_entries(values, |value, index| {
        Some(SimilarApp {
            position: index + 1,
            app_id: app_id(value)?,
            name: text(value, "title")?,
            bundle_id: text(value, "bundleId"),
            developer_name: text(value, "developerName"),
            subtitle: text(value, "subtitle"),
            rating: value.get("rating").and_then(Value::as_f64),
            rating_count_display: text(value, "ratingCount"),
            offer: value
                .pointer("/offerDisplayProperties/titles/standard")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
            is_free: value
                .pointer("/offerDisplayProperties/isFree")
                .and_then(Value::as_bool),
            has_in_app_purchases: value
                .pointer("/offerDisplayProperties/hasInAppPurchases")
                .and_then(Value::as_bool),
            app_store_url: value
                .pointer("/clickAction/pageUrl")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
            icon_url: value.get("icon").and_then(artwork_url),
        })
    })
}

fn app_id(value: &Value) -> Option<u64> {
    value
        .get("adamId")
        .and_then(|id| id.as_u64().or_else(|| id.as_str()?.parse().ok()))
}

fn artwork_url(icon: &Value) -> Option<String> {
    let template = text(icon, "template")?;
    let url = template
        .replace("{w}", "512")
        .replace("{h}", "512")
        .replace("{c}", "bb")
        .replace("{f}", "jpg");
    (!url.contains(['{', '}'])).then_some(url)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn parses_similar_app_lockups_and_preserves_shelf_position() {
        let batch = parse(&json!({
            "shelfMapping": {
                "similarItems": {
                    "items": [
                        {
                            "$kind": "Lockup",
                            "adamId": "454638411",
                            "bundleId": "com.example.one",
                            "title": "One",
                            "developerName": "Example, Inc.",
                            "subtitle": "First app",
                            "rating": 4.7,
                            "ratingCount": "13M",
                            "offerDisplayProperties": {
                                "titles": {"standard": "Get"},
                                "isFree": true,
                                "hasInAppPurchases": true
                            },
                            "clickAction": {
                                "pageUrl": "https://apps.apple.com/us/app/one/id454638411"
                            },
                            "icon": {
                                "template": "https://example.com/{w}x{h}{c}.{f}"
                            }
                        },
                        {"$kind": "Lockup", "title": "Missing ID"},
                        {
                            "$kind": "Lockup",
                            "adamId": 686449807,
                            "title": "Three",
                            "icon": {
                                "template": "https://example.com/{w}x{h}{c}.{f}"
                            }
                        }
                    ]
                }
            }
        }));

        assert_eq!(batch.skipped_count, 1);
        assert_eq!(batch.records.len(), 2);
        assert_eq!(batch.records[0].position, 1);
        assert_eq!(batch.records[0].app_id, 454638411);
        assert_eq!(
            batch.records[0].rating_count_display.as_deref(),
            Some("13M")
        );
        assert_eq!(
            batch.records[0].icon_url.as_deref(),
            Some("https://example.com/512x512bb.jpg")
        );
        assert_eq!(batch.records[1].position, 3);
        assert_eq!(batch.records[1].app_id, 686449807);
    }

    #[test]
    fn missing_similar_shelf_is_an_empty_result() {
        let batch = parse(&json!({"shelfMapping": {}}));
        assert!(batch.records.is_empty());
        assert_eq!(batch.skipped_count, 0);
    }

    #[test]
    fn unresolved_artwork_placeholders_are_not_emitted() {
        assert_eq!(
            artwork_url(&json!({"template": "https://example.com/{unknown}.jpg"})),
            None
        );
    }
}

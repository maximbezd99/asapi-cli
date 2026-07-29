use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::{json, Value};

use super::{retrieved_at, text};
use crate::{
    app_store::{product_page_url, resolve_country, ProductPagePayload},
    cli::IapArgs,
    client::ApiClient,
    output::{Envelope, Meta},
};

#[derive(Debug, Serialize, PartialEq)]
pub struct InAppPurchase {
    pub name: String,
    pub display_price: String,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct AppIaps {
    pub app_id: u64,
    pub has_in_app_purchases: bool,
    pub has_external_purchases: bool,
    pub purchases: Vec<InAppPurchase>,
}

pub async fn run(client: &ApiClient, args: &IapArgs) -> Result<Envelope> {
    let app_id = args.app.id;
    let country = resolve_country(
        args.country.country.as_deref(),
        std::slice::from_ref(&args.app),
    )?;
    let html = client
        .fetch_text(product_page_url(app_id, &country)?)
        .await?;
    let result = parse(&html, app_id)?;
    let result_count = result.purchases.len();
    Ok(Envelope {
        data: serde_json::to_value(result)?,
        meta: Meta {
            country: Some(country),
            retrieved_at: retrieved_at(),
            command: "iap".to_string(),
            source: "Apple App Store product page".to_string(),
            parameters: json!({"app_id": app_id}),
            result_count,
            skipped_count: 0,
            duplicate_count: None,
            position_note: None,
            coverage_note: Some(
                "The purchases list contains the items displayed on the public App Store page and may not be the complete catalog."
                    .to_string(),
            ),
        },
    })
}

fn parse(html: &str, app_id: u64) -> Result<AppIaps> {
    let payload = ProductPagePayload::parse(html)?;
    let data = payload.app_data(app_id)?;
    let offer = data
        .get("titleOfferDisplayProperties")
        .or_else(|| data.pointer("/lockup/offerDisplayProperties"))
        .context("App Store page does not contain purchase availability")?;
    let has_in_app_purchases = offer
        .get("hasInAppPurchases")
        .and_then(Value::as_bool)
        .context("App Store page does not report in-app purchase availability")?;
    let has_external_purchases = offer
        .get("hasExternalPurchases")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let purchases = data
        .pointer("/shelfMapping/information/items")
        .and_then(Value::as_array)
        .and_then(|items| {
            items.iter().find_map(|item| {
                let modern = item
                    .get("items_V3")
                    .and_then(Value::as_array)
                    .map(|entries| {
                        entries
                            .iter()
                            .filter(|entry| text(entry, "$kind").as_deref() == Some("textPair"))
                            .filter_map(|entry| {
                                Some(InAppPurchase {
                                    name: text(entry, "leadingText")?,
                                    display_price: text(entry, "trailingText")?,
                                })
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                if !modern.is_empty() {
                    return Some(modern);
                }
                let legacy = item
                    .pointer("/items/0/textPairs")
                    .and_then(Value::as_array)?
                    .iter()
                    .filter_map(|pair| {
                        let pair = pair.as_array()?;
                        Some(InAppPurchase {
                            name: pair.first()?.as_str()?.to_string(),
                            display_price: pair.get(1)?.as_str()?.to_string(),
                        })
                    })
                    .collect::<Vec<_>>();
                (!legacy.is_empty()).then_some(legacy)
            })
        })
        .unwrap_or_default();

    Ok(AppIaps {
        app_id,
        has_in_app_purchases,
        has_external_purchases,
        purchases,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_uses_country_product_page() {
        assert_eq!(
            product_page_url(42, "ae").unwrap().as_str(),
            "https://apps.apple.com/ae/app/id42"
        );
    }

    #[test]
    fn parses_serialized_product_page() {
        let html = r#"<html><script data-test="x" id="serialized-server-data">{
          "data": [{
            "intent": {"id": "42"},
            "data": {
              "titleOfferDisplayProperties": {
                "hasInAppPurchases": true,
                "hasExternalPurchases": false
              },
              "shelfMapping": {"information": {"items": [{
                "$kind": "Annotation",
                "items_V3": [
                  {"$kind": "textPair", "leadingText": "Pro", "trailingText": "$9.99"},
                  {"$kind": "button", "title": "Learn More"}
                ]
              }]}}
            }
          }]
        }</script></html>"#;
        let result = parse(html, 42).unwrap();
        assert!(result.has_in_app_purchases);
        assert_eq!(
            result.purchases,
            [InAppPurchase {
                name: "Pro".to_string(),
                display_price: "$9.99".to_string(),
            }]
        );
    }

    #[test]
    fn parses_app_without_iaps() {
        let html = r#"<script id="serialized-server-data">{
          "data": [{"intent": {"id": "42"}, "data": {
            "titleOfferDisplayProperties": {
              "hasInAppPurchases": false,
              "hasExternalPurchases": false
            }
          }}]
        }</script>"#;
        let result = parse(html, 42).unwrap();
        assert!(!result.has_in_app_purchases);
        assert!(result.purchases.is_empty());
    }
}

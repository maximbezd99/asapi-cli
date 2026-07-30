use anyhow::{bail, Result};
use chrono::{DateTime, FixedOffset};
use reqwest::Url;
use serde::Serialize;
use serde_json::{json, Value};

use super::{artist_id_from_href, envelope, label, parse_feed_entries, text, ParseBatch};
use crate::{
    app_store::resolve_country, categories::is_valid_category, client::ApiClient, output::Envelope,
    requests::ChartRequest,
};

#[derive(Debug, Serialize, PartialEq)]
pub struct ChartApp {
    pub rank: usize,
    pub app_id: u64,
    pub bundle_id: Option<String>,
    pub name: String,
    pub developer_name: Option<String>,
    pub developer_id: Option<u64>,
    pub primary_category_id: Option<u32>,
    pub primary_category: Option<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub icon_url: Option<String>,
    pub summary: Option<String>,
    pub app_store_url: Option<String>,
    pub released_at: Option<DateTime<FixedOffset>>,
}

pub async fn run(client: &ApiClient, args: &ChartRequest) -> Result<Envelope> {
    if !(1..=200).contains(&args.limit) {
        bail!("chart limit must be between 1 and 200");
    }
    if let Some(category) = args.category {
        if !is_valid_category(category) {
            bail!("unsupported category ID {category}; run 'asapi list categories'");
        }
    }
    let country = resolve_country(args.country.as_deref(), &[])?;
    let chart = args.chart.as_str();
    let json = client
        .fetch_json(chart_url(chart, &country, args.limit, args.category)?)
        .await?;
    let batch = parse(&json);
    envelope(
        "chart",
        "Apple App Store charts",
        Some(country),
        json!({"chart": chart, "limit": args.limit, "category_id": args.category}),
        &batch.records,
        batch.skipped_count,
        None,
        Some("One-based position in this chart at retrieval time."),
    )
}

fn chart_url(chart: &str, country: &str, limit: u16, category: Option<u32>) -> Result<Url> {
    let feed = match chart {
        "top" => "topapplications",
        "free" => "topfreeapplications",
        "paid" => "toppaidapplications",
        "grossing" => "topgrossingapplications",
        _ => bail!("unsupported chart type '{chart}'"),
    };
    let genre = category
        .map(|id| format!("/genre={id}"))
        .unwrap_or_default();
    Ok(Url::parse(&format!(
        "https://itunes.apple.com/{country}/rss/{feed}/limit={limit}{genre}/json"
    ))?)
}

fn parse(json: &Value) -> ParseBatch<ChartApp> {
    parse_feed_entries(json, |value, index| {
        let attributes = value.get("id")?.get("attributes")?;
        Some(ChartApp {
            rank: index + 1,
            app_id: text(attributes, "im:id")?.parse().ok()?,
            bundle_id: text(attributes, "im:bundleId"),
            name: label(value, &["im:name"])?,
            developer_name: label(value, &["im:artist"]),
            developer_id: value
                .get("im:artist")
                .and_then(|artist| artist.get("attributes"))
                .and_then(|attributes| text(attributes, "im:id"))
                .and_then(|id| id.parse().ok())
                .or_else(|| artist_id_from_href(value)),
            primary_category_id: value
                .get("category")
                .and_then(|category| category.get("attributes"))
                .and_then(|attributes| text(attributes, "im:id"))
                .and_then(|id| id.parse().ok()),
            primary_category: value
                .get("category")
                .and_then(|category| category.get("attributes"))
                .and_then(|attributes| text(attributes, "label")),
            price: value
                .get("im:price")
                .and_then(|price| price.get("attributes"))
                .and_then(|attributes| text(attributes, "amount"))
                .and_then(|price| price.parse().ok()),
            currency: value
                .get("im:price")
                .and_then(|price| price.get("attributes"))
                .and_then(|attributes| text(attributes, "currency")),
            icon_url: value
                .get("im:image")
                .and_then(Value::as_array)
                .and_then(|images| images.last())
                .and_then(|image| label(image, &[])),
            summary: label(value, &["summary"]),
            app_store_url: label(value, &["id"]),
            released_at: label(value, &["im:releaseDate"])
                .and_then(|date| DateTime::parse_from_rfc3339(&date).ok()),
        })
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn url_places_genre_after_limit() {
        assert_eq!(
            chart_url("free", "us", 25, Some(6007)).unwrap().as_str(),
            "https://itunes.apple.com/us/rss/topfreeapplications/limit=25/genre=6007/json"
        );
    }

    #[test]
    fn ranks_survive_filtering() {
        let batch = parse(&json!({"feed": {"entry": [
            {"id": {"attributes": {"im:id": "1"}}, "im:name": {"label": "One"}},
            {"id": {"attributes": {}}, "im:name": {"label": "Broken"}},
            {"id": {"attributes": {"im:id": "3"}}, "im:name": {"label": "Three"}}
        ]}}));
        assert_eq!(batch.skipped_count, 1);
        assert_eq!(batch.records[1].rank, 3);
    }

    #[test]
    fn parses_single_entry_object() {
        let batch = parse(&json!({"feed": {"entry": {
            "id": {"attributes": {"im:id": "1"}},
            "im:name": {"label": "One"}
        }}}));
        assert_eq!(batch.skipped_count, 0);
        assert_eq!(batch.records.len(), 1);
        assert_eq!(batch.records[0].rank, 1);
    }
}

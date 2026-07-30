use anyhow::{bail, Result};
use reqwest::Url;
use serde::Serialize;
use serde_json::{json, Value};

use super::{envelope, parse_result_array, strings, text, u32_field, ParseBatch};
use crate::{
    app_store::resolve_country, client::ApiClient, output::Envelope, requests::SearchRequest,
};

#[derive(Debug, Serialize, PartialEq)]
pub struct SearchApp {
    pub position: usize,
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
    pub description: Option<String>,
    pub categories: Vec<String>,
    pub seller_name: Option<String>,
    pub app_store_url: Option<String>,
    pub icon_url: Option<String>,
}

pub async fn run(client: &ApiClient, args: &SearchRequest) -> Result<Envelope> {
    if args.term.trim().is_empty() {
        bail!("search term must not be empty");
    }
    if !(1..=200).contains(&args.limit) {
        bail!("search limit must be between 1 and 200");
    }
    validate_local_limit(args.limit, args.local_limit)?;
    let country = resolve_country(args.country.as_deref(), &[])?;
    let json = client
        .fetch_json(search_url(&args.term, &country, args.limit)?)
        .await?;
    let mut batch = parse(&json);
    apply_local_limit(&mut batch.records, args.local_limit);
    let parameters = match args.local_limit {
        Some(local_limit) => {
            json!({"term": args.term, "limit": args.limit, "local_limit": local_limit})
        }
        None => json!({"term": args.term, "limit": args.limit}),
    };
    envelope(
        "search",
        "Apple App Store search",
        Some(country),
        parameters,
        &batch.records,
        batch.skipped_count,
        None,
        Some("One-based order in this search result set; not an App Store keyword rank."),
    )
}

fn validate_local_limit(limit: u32, local_limit: Option<u32>) -> Result<()> {
    if local_limit.is_some_and(|local_limit| !(1..=200).contains(&local_limit)) {
        bail!("local limit must be between 1 and 200");
    }
    if local_limit.is_some_and(|local_limit| local_limit > limit) {
        bail!("--local-limit must not exceed --limit");
    }
    Ok(())
}

fn apply_local_limit<T>(records: &mut Vec<T>, local_limit: Option<u32>) {
    if let Some(limit) = local_limit {
        records.truncate(limit as usize);
    }
}

fn search_url(term: &str, country: &str, limit: u32) -> Result<Url> {
    let mut url = Url::parse("https://itunes.apple.com/search")?;
    url.query_pairs_mut()
        .append_pair("term", term)
        .append_pair("country", country)
        .append_pair("media", "software")
        .append_pair("entity", "software")
        .append_pair("limit", &limit.to_string());
    Ok(url)
}

fn parse(json: &Value) -> ParseBatch<SearchApp> {
    parse_result_array(json, |value, index| {
        Some(SearchApp {
            position: index + 1,
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
            description: text(value, "description"),
            categories: strings(value, "genres"),
            seller_name: text(value, "sellerName"),
            app_store_url: text(value, "trackViewUrl"),
            icon_url: text(value, "artworkUrl512").or_else(|| text(value, "artworkUrl100")),
        })
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn query_is_url_encoded() {
        let url = search_url("notes & tasks", "us", 10).unwrap();
        assert!(url.as_str().contains("term=notes+%26+tasks"));
        assert!(url.as_str().contains("entity=software"));
    }

    #[test]
    fn positions_survive_filtering() {
        let batch = parse(&json!({"results": [
            {"trackId": 1, "trackName": "One"},
            {"trackName": "Broken"},
            {"trackId": 3, "trackName": "Three"}
        ]}));
        assert_eq!(batch.skipped_count, 1);
        assert_eq!(batch.records[1].position, 3);
    }

    #[test]
    fn local_limit_truncates_normalized_records() {
        let mut records = vec![1, 2, 3, 4];
        apply_local_limit(&mut records, Some(2));
        assert_eq!(records, vec![1, 2]);
    }

    #[test]
    fn local_limit_cannot_exceed_upstream_limit() {
        assert!(validate_local_limit(10, Some(11)).is_err());
        assert!(validate_local_limit(10, Some(10)).is_ok());
        assert!(validate_local_limit(10, None).is_ok());
    }
}

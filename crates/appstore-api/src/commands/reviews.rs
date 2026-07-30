use std::collections::HashSet;

use anyhow::{bail, Result};
use chrono::{DateTime, FixedOffset};
use reqwest::Url;
use serde::Serialize;
use serde_json::{json, Value};

use super::{envelope, label, parse_feed_entries, ParseBatch};
use crate::{
    app_store::resolve_country, client::ApiClient, output::Envelope, requests::ReviewsRequest,
};

#[derive(Debug, Serialize, PartialEq)]
pub struct AppReview {
    pub id: u64,
    pub author: Option<String>,
    pub rating: u8,
    pub title: Option<String>,
    pub content: String,
    pub version: Option<String>,
    pub helpful_score: Option<i64>,
    pub helpful_vote_count: Option<i64>,
    pub updated_at: Option<DateTime<FixedOffset>>,
}

pub async fn run(client: &ApiClient, args: &ReviewsRequest) -> Result<Envelope> {
    let selection_count = usize::from(args.page.is_some())
        + usize::from(args.pages.is_some())
        + usize::from(args.all);
    if selection_count > 1 {
        bail!("review page, pages, and all modes are mutually exclusive");
    }
    if args.page.is_some_and(|page| !(1..=10).contains(&page))
        || args.pages.is_some_and(|pages| !(1..=10).contains(&pages))
    {
        bail!("review pages must be between 1 and 10");
    }
    let app_id = args.app.id;
    let country = resolve_country(args.country.as_deref(), std::slice::from_ref(&args.app))?;
    let pages = args.requested_pages();
    let mut all = Vec::new();
    let mut skipped_count = 0;
    let mut pages_retrieved = 0;
    for page in &pages {
        let json = client
            .fetch_json(reviews_url(app_id, &country, *page)?)
            .await?;
        let batch = parse(&json);
        skipped_count += batch.skipped_count;
        pages_retrieved += 1;
        all.extend(batch.records);
    }
    let (reviews, duplicate_count) = deduplicate(all);
    let parameters = json!({
        "app_id": app_id,
        "pages_requested": pages,
        "pages_retrieved": pages_retrieved,
        "maximum_available_pages": 10
    });
    envelope(
        "reviews",
        "Apple App Store customer reviews",
        Some(country),
        parameters,
        &reviews,
        skipped_count,
        Some(duplicate_count),
        None,
    )
}

fn reviews_url(id: u64, country: &str, page: u8) -> Result<Url> {
    Ok(Url::parse(&format!(
        "https://itunes.apple.com/{country}/rss/customerreviews/page={page}/id={id}/sortby=mostrecent/json"
    ))?)
}

fn parse(json: &Value) -> ParseBatch<AppReview> {
    parse_feed_entries(json, |value, _| {
        let id = label(value, &["id"])?.parse().ok()?;
        let rating = label(value, &["im:rating"])?.parse::<u8>().ok()?;
        if !(1..=5).contains(&rating) {
            return None;
        }
        Some(AppReview {
            id,
            author: label(value, &["author", "name"]),
            rating,
            title: label(value, &["title"]),
            content: label(value, &["content"]).unwrap_or_default(),
            version: label(value, &["im:version"]),
            helpful_score: label(value, &["im:voteSum"]).and_then(|number| number.parse().ok()),
            helpful_vote_count: label(value, &["im:voteCount"])
                .and_then(|number| number.parse().ok()),
            updated_at: label(value, &["updated"])
                .and_then(|date| DateTime::parse_from_rfc3339(&date).ok()),
        })
    })
}

fn deduplicate(reviews: Vec<AppReview>) -> (Vec<AppReview>, usize) {
    let original_count = reviews.len();
    let mut ids = HashSet::new();
    let unique = reviews
        .into_iter()
        .filter(|review| ids.insert(review.id))
        .collect::<Vec<_>>();
    let duplicate_count = original_count - unique.len();
    (unique, duplicate_count)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn review(id: u64) -> AppReview {
        AppReview {
            id,
            author: None,
            rating: 5,
            title: None,
            content: String::new(),
            version: None,
            helpful_score: None,
            helpful_vote_count: None,
            updated_at: None,
        }
    }

    #[test]
    fn deduplication_keeps_first_occurrence() {
        let (reviews, duplicates) = deduplicate(vec![review(1), review(2), review(1)]);
        assert_eq!(
            reviews.iter().map(|review| review.id).collect::<Vec<_>>(),
            [1, 2]
        );
        assert_eq!(duplicates, 1);
    }

    #[test]
    fn rejects_invalid_rating() {
        let batch = parse(&json!({"feed": {"entry": [
            {"id": {"label": "7"}, "im:rating": {"label": "5"}, "content": {"label": "Good"}},
            {"id": {"label": "8"}, "im:rating": {"label": "0"}}
        ]}}));
        assert_eq!(batch.records[0].id, 7);
        assert_eq!(batch.skipped_count, 1);
    }

    #[test]
    fn parses_single_entry_object() {
        let batch = parse(&json!({"feed": {"entry": {
            "id": {"label": "7"},
            "im:rating": {"label": "5"},
            "content": {"label": "Good"}
        }}}));
        assert_eq!(batch.skipped_count, 0);
        assert_eq!(batch.records.len(), 1);
        assert_eq!(batch.records[0].id, 7);
    }
}

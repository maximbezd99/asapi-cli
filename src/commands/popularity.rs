use std::collections::HashSet;

use anyhow::{bail, Result};
use serde::Serialize;
use serde_json::json;

use super::{envelope, lookup};
use crate::{
    cli::{PopularityArgs, PopularityGroup},
    client::ApiClient,
    countries::validate_country,
    output::Envelope,
};

const TIER1_COUNTRIES: &[&str] = &["us", "ca", "cn", "jp", "gb", "de", "fr", "kr", "au"];
const TIER2_COUNTRIES: &[&str] = &[
    "in", "br", "mx", "es", "it", "nl", "id", "sg", "hk", "tw", "ae",
];

#[derive(Debug, Serialize, PartialEq)]
pub struct PopularityRecord {
    pub app_id: u64,
    pub country: String,
    pub available: bool,
    pub name: Option<String>,
    pub rating: Option<f64>,
    pub rating_count: Option<u64>,
}

pub async fn run(client: &ApiClient, args: &PopularityArgs) -> Result<Envelope> {
    if args.id == 0 {
        bail!("app ID must be a positive integer");
    }

    let countries = selected_countries(args)?;
    let mut records = Vec::with_capacity(countries.len());
    let mut skipped_count = 0;

    for country in &countries {
        let json = client
            .fetch_json(lookup::lookup_url(&[args.id], country)?)
            .await?;
        let batch = lookup::parse(&json);
        skipped_count += batch.skipped_count;
        records.push(record_for_country(
            args.id,
            country.clone(),
            batch.records.into_iter().next(),
        ));
    }

    sort_records(&mut records);

    let selected_group = args
        .countries
        .is_none()
        .then(|| args.group.as_str().to_string());
    let mut result = envelope(
        "popularity",
        "Apple App Store",
        None,
        json!({
            "app_id": args.id,
            "group": selected_group,
            "countries": countries,
        }),
        &records,
        skipped_count,
        None,
        None,
    )?;
    result.meta.coverage_note = Some(
        "Country-specific rating_count is a public popularity signal, not a download, revenue, or active-user estimate."
            .to_string(),
    );
    Ok(result)
}

fn sort_records(records: &mut [PopularityRecord]) {
    records.sort_by(|left, right| {
        right
            .rating_count
            .cmp(&left.rating_count)
            .then_with(|| left.country.cmp(&right.country))
    });
}

fn selected_countries(args: &PopularityArgs) -> Result<Vec<String>> {
    let values: Vec<&str> = match &args.countries {
        Some(countries) => countries.iter().map(String::as_str).collect(),
        None => match args.group {
            PopularityGroup::Tier1 => TIER1_COUNTRIES.to_vec(),
            PopularityGroup::Tier2 => TIER1_COUNTRIES
                .iter()
                .chain(TIER2_COUNTRIES)
                .copied()
                .collect(),
        },
    };

    let mut seen = HashSet::new();
    let mut countries = Vec::with_capacity(values.len());
    for value in values {
        let country = validate_country(value)?;
        if seen.insert(country.clone()) {
            countries.push(country);
        }
    }
    Ok(countries)
}

fn record_for_country(
    app_id: u64,
    country: String,
    app: Option<lookup::LookupApp>,
) -> PopularityRecord {
    match app {
        Some(app) => PopularityRecord {
            app_id,
            country,
            available: true,
            name: Some(app.name),
            rating: app.rating,
            rating_count: app.rating_count,
        },
        None => PopularityRecord {
            app_id,
            country,
            available: false,
            name: None,
            rating: None,
            rating_count: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(group: PopularityGroup, countries: Option<Vec<&str>>) -> PopularityArgs {
        PopularityArgs {
            id: 42,
            group,
            countries: countries.map(|values| values.into_iter().map(str::to_string).collect()),
        }
    }

    #[test]
    fn tier1_selects_nine_countries() {
        let countries = selected_countries(&args(PopularityGroup::Tier1, None)).unwrap();
        assert_eq!(countries, TIER1_COUNTRIES);
        assert_eq!(countries.len(), 9);
    }

    #[test]
    fn tier2_includes_tier1_and_tier2_countries() {
        let countries = selected_countries(&args(PopularityGroup::Tier2, None)).unwrap();
        assert_eq!(countries.len(), 20);
        assert_eq!(&countries[..TIER1_COUNTRIES.len()], TIER1_COUNTRIES);
        assert_eq!(&countries[TIER1_COUNTRIES.len()..], TIER2_COUNTRIES);
    }

    #[test]
    fn explicit_countries_override_group_and_are_normalized_and_deduplicated() {
        let countries = selected_countries(&args(
            PopularityGroup::Tier2,
            Some(vec![" JP ", "us", "jp", "GB"]),
        ))
        .unwrap();
        assert_eq!(countries, ["jp", "us", "gb"]);
    }

    #[test]
    fn explicit_countries_are_validated() {
        let error =
            selected_countries(&args(PopularityGroup::Tier1, Some(vec!["xx"]))).unwrap_err();
        assert!(error
            .to_string()
            .contains("unsupported App Store country code"));
    }

    #[test]
    fn records_are_sorted_by_rating_count_with_missing_counts_last() {
        let mut records = vec![
            PopularityRecord {
                app_id: 42,
                country: "us".into(),
                available: true,
                name: Some("App".into()),
                rating: Some(4.0),
                rating_count: Some(10),
            },
            PopularityRecord {
                app_id: 42,
                country: "gb".into(),
                available: false,
                name: None,
                rating: None,
                rating_count: None,
            },
            PopularityRecord {
                app_id: 42,
                country: "jp".into(),
                available: true,
                name: Some("App".into()),
                rating: Some(4.5),
                rating_count: Some(30),
            },
        ];

        sort_records(&mut records);

        assert_eq!(
            records
                .iter()
                .map(|record| record.country.as_str())
                .collect::<Vec<_>>(),
            ["jp", "us", "gb"]
        );
    }
}

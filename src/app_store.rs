use std::{borrow::Cow, str::FromStr};

use anyhow::{bail, Context, Result};
use reqwest::Url;
use serde_json::Value;

use crate::countries::validate_country;

const DEFAULT_COUNTRY: &str = "us";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSpecifier {
    pub id: u64,
    pub country: Option<String>,
}

pub(crate) struct ProductPagePayload {
    payload: Value,
}

impl ProductPagePayload {
    pub(crate) fn parse(html: &str) -> Result<Self> {
        let marker = "id=\"serialized-server-data\"";
        let marker_start = html
            .find(marker)
            .context("App Store page does not contain product data")?;
        let content_start = html[marker_start..]
            .find('>')
            .map(|offset| marker_start + offset + 1)
            .context("App Store product data is malformed")?;
        let content_end = html[content_start..]
            .find("</script>")
            .map(|offset| content_start + offset)
            .context("App Store product data is incomplete")?;
        let payload = serde_json::from_str(&html[content_start..content_end])
            .context("App Store product data is invalid JSON")?;
        Ok(Self { payload })
    }

    pub(crate) fn app_data(&self, app_id: u64) -> Result<&Value> {
        let expected_id = app_id.to_string();
        self.payload
            .get("data")
            .and_then(Value::as_array)
            .and_then(|pages| {
                pages.iter().find(|page| {
                    page.pointer("/intent/id").and_then(Value::as_str) == Some(expected_id.as_str())
                })
            })
            .with_context(|| format!("App Store page does not contain app {app_id}"))?
            .get("data")
            .context("App Store page does not contain app details")
    }
}

impl FromStr for AppSpecifier {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        parse_app_specifier(value).map_err(|error| error.to_string())
    }
}

pub(crate) fn product_page_url(id: u64, country: &str) -> Result<Url> {
    Ok(Url::parse(&format!(
        "https://apps.apple.com/{country}/app/id{id}"
    ))?)
}

pub fn resolve_country(explicit: Option<&str>, apps: &[AppSpecifier]) -> Result<String> {
    if let Some(country) = explicit {
        return validate_country(country);
    }

    let mut countries = apps
        .iter()
        .filter_map(|app| app.country.as_deref())
        .map(validate_country)
        .collect::<Result<Vec<_>>>()?;
    countries.sort();
    countries.dedup();

    match countries.as_slice() {
        [] => Ok(DEFAULT_COUNTRY.to_string()),
        [country] => Ok(country.clone()),
        _ => bail!(
            "App Store URLs use multiple countries ({}); pass --country to choose one",
            countries.join(", ")
        ),
    }
}

fn parse_app_specifier(value: &str) -> Result<AppSpecifier> {
    let value = value.trim();
    if let Ok(id) = value.parse::<u64>() {
        if id == 0 {
            bail!("app ID must be a positive integer");
        }
        return Ok(AppSpecifier { id, country: None });
    }

    let url_value = normalize_app_store_url(value);
    let url = Url::parse(&url_value).map_err(|_| {
        anyhow::anyhow!("expected a positive App Store ID or App Store product URL")
    })?;
    if url.scheme() != "https" {
        bail!("App Store URL must use https");
    }
    let host = url.host_str().unwrap_or_default();
    let host = host.strip_prefix("www.").unwrap_or(host);
    if !matches!(host, "apps.apple.com" | "itunes.apple.com") {
        bail!("URL must be an apps.apple.com or itunes.apple.com product URL");
    }

    let segments = url
        .path_segments()
        .map(Iterator::collect::<Vec<_>>)
        .unwrap_or_default();
    let app_index = segments
        .iter()
        .position(|segment| segment.eq_ignore_ascii_case("app"))
        .ok_or_else(|| anyhow::anyhow!("App Store URL does not contain an app product path"))?;
    let id = segments[app_index + 1..]
        .iter()
        .rev()
        .find_map(|segment| segment.strip_prefix("id"))
        .and_then(|id| id.parse::<u64>().ok())
        .filter(|id| *id > 0)
        .ok_or_else(|| anyhow::anyhow!("App Store URL does not contain a positive app ID"))?;

    let country = match app_index {
        0 => None,
        1 => {
            let country = segments[0];
            if country.len() != 2 || !country.bytes().all(|byte| byte.is_ascii_alphabetic()) {
                bail!("App Store URL contains an invalid country storefront");
            }
            Some(country.to_ascii_lowercase())
        }
        _ => bail!("App Store URL contains an unsupported product path"),
    };

    Ok(AppSpecifier { id, country })
}

fn normalize_app_store_url(value: &str) -> Cow<'_, str> {
    let without_prefix = value.strip_prefix("//").unwrap_or(value);
    let host = without_prefix
        .split('/')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let host = host.strip_prefix("www.").unwrap_or(&host);
    if matches!(host, "apps.apple.com" | "itunes.apple.com") {
        if value.starts_with("//") {
            Cow::Owned(format!("https:{value}"))
        } else {
            Cow::Owned(format!("https://{value}"))
        }
    } else {
        Cow::Borrowed(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(value: &str) -> AppSpecifier {
        value.parse().unwrap()
    }

    #[test]
    fn parses_numeric_id() {
        assert_eq!(
            app("123456789"),
            AppSpecifier {
                id: 123456789,
                country: None,
            }
        );
    }

    #[test]
    fn parses_modern_product_url() {
        assert_eq!(
            app("https://apps.apple.com/ae/app/example/id123456789?platform=iphone"),
            AppSpecifier {
                id: 123456789,
                country: Some("ae".to_string()),
            }
        );
    }

    #[test]
    fn parses_product_urls_without_a_scheme() {
        let expected = AppSpecifier {
            id: 6752595210,
            country: Some("ae".to_string()),
        };
        assert_eq!(app("apps.apple.com/ae/app/verento/id6752595210"), expected);
        assert_eq!(
            app("//apps.apple.com/ae/app/verento/id6752595210"),
            expected
        );
    }

    #[test]
    fn parses_countryless_and_legacy_product_urls() {
        assert_eq!(app("https://apps.apple.com/app/example/id42").country, None);
        assert_eq!(
            app("https://itunes.apple.com/GB/app/example/id42?mt=8").country,
            Some("gb".to_string())
        );
    }

    #[test]
    fn rejects_invalid_inputs() {
        for value in [
            "0",
            "not-an-app",
            "example.com/us/app/example/id42",
            "http://apps.apple.com/us/app/example/id42",
            "https://example.com/us/app/example/id42",
            "https://apps.apple.com/us/developer/example/id42",
            "https://apps.apple.com/us/app/example",
        ] {
            assert!(value.parse::<AppSpecifier>().is_err(), "{value}");
        }
    }

    #[test]
    fn explicit_country_takes_priority() {
        let apps = [
            app("https://apps.apple.com/gb/app/one/id1"),
            app("https://apps.apple.com/jp/app/two/id2"),
        ];
        assert_eq!(resolve_country(Some(" AE "), &apps).unwrap(), "ae");
    }

    #[test]
    fn explicit_country_can_override_an_unsupported_link_storefront() {
        let apps = [app("https://apps.apple.com/zz/app/one/id1")];
        assert_eq!(resolve_country(Some("us"), &apps).unwrap(), "us");
        assert!(resolve_country(None, &apps).is_err());
    }

    #[test]
    fn link_country_is_used_without_an_explicit_country() {
        let apps = [app("42"), app("https://apps.apple.com/gb/app/one/id1")];
        assert_eq!(resolve_country(None, &apps).unwrap(), "gb");
    }

    #[test]
    fn raw_ids_default_to_us() {
        assert_eq!(resolve_country(None, &[app("42")]).unwrap(), "us");
    }

    #[test]
    fn conflicting_link_countries_require_an_override() {
        let apps = [
            app("https://apps.apple.com/gb/app/one/id1"),
            app("https://apps.apple.com/jp/app/two/id2"),
        ];
        assert!(resolve_country(None, &apps)
            .unwrap_err()
            .to_string()
            .contains("pass --country"));
    }
}

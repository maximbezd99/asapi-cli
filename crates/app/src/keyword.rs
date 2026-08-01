use anyhow::{bail, Result};
use appstore_api::countries::validate_country;

const MAX_KEYWORD_LENGTH: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedKeyword(String);

impl NormalizedKeyword {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeywordIdentity {
    display: String,
    normalized: NormalizedKeyword,
    country: String,
}

impl KeywordIdentity {
    pub fn new(keyword: &str, country: &str) -> Result<Self> {
        let display = standardize_display(keyword)?;
        let normalized = NormalizedKeyword(display.to_lowercase());
        let country = validate_country(country)?;
        Ok(Self {
            display,
            normalized,
            country,
        })
    }

    pub fn display(&self) -> &str {
        &self.display
    }

    pub fn normalized(&self) -> &NormalizedKeyword {
        &self.normalized
    }

    pub fn country(&self) -> &str {
        &self.country
    }
}

fn standardize_display(keyword: &str) -> Result<String> {
    let display = keyword.split_whitespace().collect::<Vec<_>>().join(" ");
    if display.is_empty() {
        bail!("keyword must not be empty");
    }
    if display.chars().count() > MAX_KEYWORD_LENGTH {
        bail!("keyword must be at most {MAX_KEYWORD_LENGTH} characters");
    }
    Ok(display)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_collapses_whitespace_and_normalizes_case_and_country() {
        let identity = KeywordIdentity::new("  App\n  Store  ", "US").unwrap();
        assert_eq!(identity.display(), "App Store");
        assert_eq!(identity.normalized().as_str(), "app store");
        assert_eq!(identity.country(), "us");
    }

    #[test]
    fn identity_rejects_empty_and_overlong_keywords() {
        assert!(KeywordIdentity::new(" \n ", "us").is_err());
        assert!(KeywordIdentity::new(&"a".repeat(201), "us").is_err());
    }
}

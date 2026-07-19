use std::time::Duration;

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use reqwest::{header::RETRY_AFTER, Response, StatusCode, Url};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub timeout: Duration,
    pub retries: u8,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            retries: 2,
        }
    }
}

pub struct ApiClient {
    http: reqwest::Client,
    retries: u8,
}

impl ApiClient {
    pub fn new(config: ClientConfig) -> Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(config.timeout)
            .user_agent(concat!("asapi/", env!("CARGO_PKG_VERSION")))
            .build()
            .context("failed to build HTTP client")?;
        Ok(Self {
            http,
            retries: config.retries,
        })
    }

    pub(crate) async fn fetch_json(&self, url: Url) -> Result<Value> {
        let endpoint = endpoint_name(&url);
        self.fetch_response(url)
            .await?
            .json()
            .await
            .with_context(|| format!("invalid JSON response from {endpoint}"))
    }

    pub(crate) async fn fetch_text(&self, url: Url) -> Result<String> {
        let endpoint = endpoint_name(&url);
        self.fetch_response(url)
            .await?
            .text()
            .await
            .with_context(|| format!("invalid text response from {endpoint}"))
    }

    async fn fetch_response(&self, url: Url) -> Result<Response> {
        let endpoint = endpoint_name(&url);
        let max_attempts = u16::from(self.retries) + 1;
        for attempt in 1..=max_attempts {
            match self.http.get(url.clone()).send().await {
                Ok(response) if response.status().is_success() => {
                    return Ok(response);
                }
                Ok(response) => {
                    let status = response.status();
                    let retry_after = retry_after_delay(&response, Utc::now());
                    if !retryable_status(status) || attempt == max_attempts {
                        bail!(
                            "request to {endpoint} failed with HTTP {status} after {}. {}",
                            attempt_label(attempt),
                            http_recommendation(status, retry_after)
                        );
                    }
                    let delay = retry_after.unwrap_or_else(|| backoff_delay(attempt));
                    eprintln!(
                        "request returned HTTP {status}; retrying in {} ({attempt}/{max_attempts})",
                        display_delay(delay)
                    );
                    tokio::time::sleep(delay).await;
                }
                Err(error) => {
                    if !retryable_request_error(&error) {
                        bail!(
                            "request to {endpoint} failed after {}: {error}. Check the command arguments and try again.",
                            attempt_label(attempt)
                        );
                    }
                    if attempt == max_attempts {
                        bail!(
                            "request to {endpoint} failed after {}: {error}. Check network connectivity or try again later.",
                            attempt_label(attempt)
                        );
                    }
                    let delay = backoff_delay(attempt);
                    eprintln!(
                        "request failed: {error}; retrying in {} ({attempt}/{max_attempts})",
                        display_delay(delay)
                    );
                    tokio::time::sleep(delay).await;
                }
            }
        }
        unreachable!("request loop always returns")
    }
}

fn endpoint_name(url: &Url) -> String {
    format!(
        "{}://{}{}",
        url.scheme(),
        url.host_str().unwrap_or("unknown"),
        url.path()
    )
}

fn retryable_status(status: StatusCode) -> bool {
    status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
}

fn retryable_request_error(error: &reqwest::Error) -> bool {
    error.is_timeout() || error.is_connect() || error.is_request()
}

fn backoff_delay(attempt: u16) -> Duration {
    Duration::from_millis(250 * 2_u64.pow(u32::from(attempt - 1)))
}

fn retry_after_delay(response: &Response, now: DateTime<Utc>) -> Option<Duration> {
    let value = response.headers().get(RETRY_AFTER)?.to_str().ok()?;
    parse_retry_after(value, now)
}

fn parse_retry_after(value: &str, now: DateTime<Utc>) -> Option<Duration> {
    if let Ok(seconds) = value.trim().parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }
    let retry_at = DateTime::parse_from_rfc2822(value)
        .ok()?
        .with_timezone(&Utc);
    retry_at.signed_duration_since(now).to_std().ok()
}

fn http_recommendation(status: StatusCode, retry_after: Option<Duration>) -> String {
    match status {
        StatusCode::BAD_REQUEST => {
            "Check the command arguments and supported country or category values.".to_string()
        }
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
            "Apple denied the request; verify that this public resource is available for the selected country."
                .to_string()
        }
        StatusCode::NOT_FOUND => {
            "Verify the app ID and country, then try again.".to_string()
        }
        StatusCode::TOO_MANY_REQUESTS => retry_after.map_or_else(
            || "Apple is rate limiting requests; wait before trying again.".to_string(),
            |delay| {
                format!(
                    "Apple is rate limiting requests; wait at least {} before trying again.",
                    display_delay(delay)
                )
            },
        ),
        status if status.is_server_error() => {
            "Apple's service is temporarily unavailable; try again later.".to_string()
        }
        _ => "The request was rejected; check the input before trying again.".to_string(),
    }
}

fn attempt_label(attempts: u16) -> String {
    if attempts == 1 {
        "1 attempt".to_string()
    } else {
        format!("{attempts} attempts")
    }
}

fn display_delay(delay: Duration) -> String {
    if delay.subsec_millis() == 0 {
        format!("{}s", delay.as_secs())
    } else {
        format!("{}ms", delay.as_millis())
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn retry_after_accepts_seconds() {
        let now = Utc.timestamp_opt(0, 0).unwrap();
        assert_eq!(parse_retry_after("12", now), Some(Duration::from_secs(12)));
    }

    #[test]
    fn retry_after_accepts_http_date() {
        let now = Utc.with_ymd_and_hms(2015, 10, 21, 7, 27, 30).unwrap();
        assert_eq!(
            parse_retry_after("Wed, 21 Oct 2015 07:28:00 GMT", now),
            Some(Duration::from_secs(30))
        );
    }

    #[test]
    fn retry_policy_only_accepts_transient_statuses() {
        assert!(retryable_status(StatusCode::TOO_MANY_REQUESTS));
        assert!(retryable_status(StatusCode::SERVICE_UNAVAILABLE));
        assert!(!retryable_status(StatusCode::BAD_REQUEST));
        assert!(!retryable_status(StatusCode::NOT_FOUND));
    }

    #[test]
    fn exhausted_errors_recommend_a_next_action() {
        assert!(http_recommendation(StatusCode::NOT_FOUND, None).contains("app ID"));
        assert!(
            http_recommendation(StatusCode::TOO_MANY_REQUESTS, Some(Duration::from_secs(20)))
                .contains("20s")
        );
        assert!(http_recommendation(StatusCode::BAD_GATEWAY, None).contains("try again later"));
    }
}

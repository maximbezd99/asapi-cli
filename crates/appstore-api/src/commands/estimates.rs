use anyhow::{bail, Result};
use serde_json::json;

use super::envelope;
use crate::{client::ApiClient, market_estimates, output::Envelope, requests::EstimatesRequest};

const MAX_REQUESTED_APPS: usize = 200;

pub async fn run(client: &ApiClient, args: &EstimatesRequest) -> Result<Envelope> {
    validate_app_ids(&args.app_ids)?;
    let mut app_ids = args.app_ids.clone();
    app_ids.sort_unstable();
    app_ids.dedup();
    let estimates = market_estimates::fetch_app_estimates(client, &app_ids).await?;
    let mut result = envelope(
        "estimates",
        "Third-party market estimates",
        None,
        json!({"app_ids": app_ids}),
        &estimates,
        0,
        None,
        None,
    )?;
    result.meta.coverage_note = Some(
        "Worldwide last-month downloads and revenue are rounded estimates; preserve the displayed string because '<' buckets are upper-bound ranges, not exact values."
            .to_string(),
    );
    Ok(result)
}

fn validate_app_ids(app_ids: &[u64]) -> Result<()> {
    if app_ids.is_empty() {
        bail!("estimates require at least one app ID");
    }
    if app_ids.len() > MAX_REQUESTED_APPS {
        bail!("estimates accept at most {MAX_REQUESTED_APPS} app IDs per request");
    }
    if app_ids.contains(&0) {
        bail!("app IDs must be positive integers");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimate_batch_bounds_are_validated() {
        assert!(validate_app_ids(&[]).is_err());
        assert!(validate_app_ids(&[0]).is_err());
        assert!(validate_app_ids(&[1, 2, 3]).is_ok());
        assert!(validate_app_ids(&vec![1; MAX_REQUESTED_APPS + 1]).is_err());
    }
}

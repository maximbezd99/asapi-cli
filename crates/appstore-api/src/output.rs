use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct Envelope {
    pub data: Value,
    pub meta: Meta,
}

#[derive(Debug, Serialize)]
pub struct Meta {
    pub country: Option<String>,
    pub retrieved_at: String,
    pub command: String,
    pub source: String,
    pub parameters: Value,
    pub result_count: usize,
    pub skipped_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duplicate_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage_note: Option<String>,
}

pub fn emit<T: Serialize + ?Sized>(
    value: &T,
    pretty: bool,
    destination: Option<&Path>,
) -> Result<()> {
    let rendered = if pretty {
        serde_json::to_string_pretty(value)?
    } else {
        serde_json::to_string(value)?
    };
    if let Some(path) = destination {
        fs::write(path, format!("{rendered}\n"))
            .with_context(|| format!("failed to write {}", path.display()))?;
    } else {
        println!("{rendered}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn envelope_serializes_data_and_provenance() {
        let envelope = Envelope {
            data: json!([{"app_id": 1}]),
            meta: Meta {
                country: Some("us".to_string()),
                retrieved_at: "2026-01-01T00:00:00Z".to_string(),
                command: "lookup".to_string(),
                source: "Apple Search API".to_string(),
                parameters: json!({"app_ids": [1]}),
                result_count: 1,
                skipped_count: 0,
                duplicate_count: None,
                position_note: None,
                coverage_note: None,
            },
        };
        let value = serde_json::to_value(&envelope).unwrap();
        assert_eq!(value["data"][0]["app_id"], 1);
        assert_eq!(value["meta"]["country"], "us");
    }
}

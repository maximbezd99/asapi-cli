use std::path::Path;

use anyhow::Result;
use chrono::{DateTime, FixedOffset, SecondsFormat, Utc};
use serde::Serialize;
use serde_json::Value;

use crate::{
    cli::Command,
    client::{ApiClient, ClientConfig},
    output::{emit, Envelope, Meta},
};

pub mod chart;
pub mod iap;
pub mod install_skill;
pub mod list;
pub mod lookup;
pub mod popularity;
pub mod reviews;
pub mod search;
pub mod similar;

pub enum CommandOutput {
    None,
    Envelope(Box<Envelope>),
    Value(Value),
}

impl CommandOutput {
    pub fn emit(self, pretty: bool, destination: Option<&Path>) -> Result<()> {
        match self {
            Self::None => Ok(()),
            Self::Envelope(value) => emit(&value, pretty, destination),
            Self::Value(value) => emit(&value, pretty, destination),
        }
    }
}

pub async fn execute(command: &Command, config: ClientConfig) -> Result<CommandOutput> {
    match command {
        Command::InstallSkill => {
            install_skill::run()?;
            return Ok(CommandOutput::None);
        }
        Command::List(args) => return Ok(CommandOutput::Value(list::run(args)?)),
        _ => {}
    }

    let client = ApiClient::new(config)?;
    let envelope = match command {
        Command::Search(args) => search::run(&client, args).await?,
        Command::Lookup(args) => lookup::run(&client, args).await?,
        Command::Popularity(args) => popularity::run(&client, args).await?,
        Command::Iap(args) => iap::run(&client, args).await?,
        Command::Similar(args) => similar::run(&client, args).await?,
        Command::Reviews(args) => reviews::run(&client, args).await?,
        Command::Chart(args) => chart::run(&client, args).await?,
        Command::InstallSkill | Command::List(_) => unreachable!("handled above"),
    };
    Ok(CommandOutput::Envelope(Box::new(envelope)))
}

#[derive(Debug, PartialEq)]
struct ParseBatch<T> {
    pub records: Vec<T>,
    pub skipped_count: usize,
}

#[allow(clippy::too_many_arguments)]
fn envelope<T: Serialize>(
    command: &str,
    source: &str,
    country: Option<String>,
    parameters: Value,
    data: &[T],
    skipped_count: usize,
    duplicate_count: Option<usize>,
    position_note: Option<&str>,
) -> Result<Envelope> {
    if skipped_count > 0 {
        eprintln!("warning: skipped {skipped_count} malformed record(s)");
    }
    Ok(Envelope {
        data: serde_json::to_value(data)?,
        meta: Meta {
            country,
            retrieved_at: retrieved_at(),
            command: command.to_string(),
            source: source.to_string(),
            parameters,
            result_count: data.len(),
            skipped_count,
            duplicate_count,
            position_note: position_note.map(ToOwned::to_owned),
            coverage_note: None,
        },
    })
}

fn retrieved_at() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn parse_result_array<T>(
    json: &Value,
    normalize: impl Fn(&Value, usize) -> Option<T>,
) -> ParseBatch<T> {
    let values = json
        .get("results")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    parse_entries(values, normalize)
}

fn parse_entries<T>(
    values: &[Value],
    normalize: impl Fn(&Value, usize) -> Option<T>,
) -> ParseBatch<T> {
    let mut records = Vec::new();
    for (index, value) in values.iter().enumerate() {
        if let Some(record) = normalize(value, index) {
            records.push(record);
        }
    }
    ParseBatch {
        skipped_count: values.len() - records.len(),
        records,
    }
}

fn parse_feed_entries<T>(
    json: &Value,
    normalize: impl Fn(&Value, usize) -> Option<T>,
) -> ParseBatch<T> {
    let values = match json.pointer("/feed/entry") {
        Some(Value::Array(values)) => values.as_slice(),
        Some(value @ Value::Object(_)) => std::slice::from_ref(value),
        _ => &[],
    };
    parse_entries(values, normalize)
}

fn text(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .filter(|text| !text.trim().is_empty())
}

fn strings(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn u32_field(value: &Value, key: &str) -> Option<u32> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|number| u32::try_from(number).ok())
}

fn date(value: &Value, key: &str) -> Option<DateTime<FixedOffset>> {
    text(value, key).and_then(|date| DateTime::parse_from_rfc3339(&date).ok())
}

fn label(value: &Value, path: &[&str]) -> Option<String> {
    let node = path.iter().try_fold(value, |node, key| node.get(key))?;
    node.get("label")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn artist_id_from_href(value: &Value) -> Option<u64> {
    let href = value
        .get("im:artist")?
        .get("attributes")?
        .get("href")?
        .as_str()?;
    href.rsplit_once("/id")?.1.split('?').next()?.parse().ok()
}

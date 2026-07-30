use anyhow::Result;
use serde_json::{json, Value};

use crate::{categories::CATEGORIES, countries::COUNTRIES, requests::ListResource};

pub fn run(resource: ListResource) -> Result<Value> {
    match resource {
        ListResource::Countries => Ok(serde_json::to_value(COUNTRIES)?),
        ListResource::Categories => Ok(serde_json::to_value(CATEGORIES)?),
        ListResource::ChartTypes => Ok(json!(["top", "free", "paid", "grossing"])),
    }
}

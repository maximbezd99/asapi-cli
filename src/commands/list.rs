use anyhow::Result;
use serde_json::{json, Value};

use crate::{
    categories::CATEGORIES,
    cli::{ListArgs, ListResource},
    countries::COUNTRIES,
};

pub fn run(args: &ListArgs) -> Result<Value> {
    match args.resource {
        ListResource::Countries => Ok(serde_json::to_value(COUNTRIES)?),
        ListResource::Categories => Ok(serde_json::to_value(CATEGORIES)?),
        ListResource::ChartTypes => Ok(json!(["top", "free", "paid", "grossing"])),
    }
}

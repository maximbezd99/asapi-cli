use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde::Serialize;

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

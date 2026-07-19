use std::{process::ExitCode, time::Duration};

use anyhow::Result;
use clap::Parser;

use asapi::{cli::Cli, client::ClientConfig, commands};

#[tokio::main]
async fn main() -> ExitCode {
    match run(Cli::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}

async fn run(cli: Cli) -> Result<()> {
    commands::execute(
        &cli.command,
        ClientConfig {
            timeout: Duration::from_secs(cli.timeout),
            retries: cli.retries,
        },
    )
    .await?
    .emit(cli.pretty, cli.output_file.as_deref())
}

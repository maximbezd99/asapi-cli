mod cli;
mod install_skill;
mod output;

use std::{process::ExitCode, time::Duration};

use anyhow::Result;
use appstore_api::{
    commands,
    requests::{
        ChartRequest, ListResource, LookupRequest, PopularityGroup, PopularityRequest,
        ReviewsRequest, SearchRequest,
    },
    ApiClient, ClientConfig,
};
use asapi_server::ServeConfig;
use clap::Parser;
use cli::{AppCommand, ChartType, Cli, Command};

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
    if matches!(cli.command, Command::InstallSkill) {
        return install_skill::run();
    }
    if let Command::App(app) = &cli.command {
        return match &app.command {
            AppCommand::Serve(args) => {
                asapi_server::serve(ServeConfig {
                    host: args.host,
                    port: args.port,
                    storage_base: args.storage_path.clone(),
                    client: client_config(&cli),
                })
                .await
            }
        };
    }

    let value = match &cli.command {
        Command::List(args) => commands::list::run(match args.resource {
            cli::ListResource::Countries => ListResource::Countries,
            cli::ListResource::Categories => ListResource::Categories,
            cli::ListResource::ChartTypes => ListResource::ChartTypes,
        })?,
        command => {
            let client = ApiClient::new(client_config(&cli))?;
            let envelope = match command {
                Command::Search(args) => {
                    commands::search::run(
                        &client,
                        &SearchRequest {
                            term: args.term.clone(),
                            country: args.country.country.clone(),
                            limit: args.limit,
                            local_limit: args.local_limit,
                        },
                    )
                    .await?
                }
                Command::Lookup(args) => {
                    commands::lookup::run(
                        &client,
                        &LookupRequest {
                            apps: args.apps.clone(),
                            country: args.country.country.clone(),
                            full: args.full,
                        },
                    )
                    .await?
                }
                Command::Popularity(args) => {
                    commands::popularity::run(
                        &client,
                        &PopularityRequest {
                            app: args.app.clone(),
                            group: match args.group {
                                cli::PopularityGroup::Tier1 => PopularityGroup::Tier1,
                                cli::PopularityGroup::Tier2 => PopularityGroup::Tier2,
                            },
                            countries: args.countries.clone(),
                        },
                    )
                    .await?
                }
                Command::Reviews(args) => {
                    commands::reviews::run(
                        &client,
                        &ReviewsRequest {
                            app: args.app.clone(),
                            country: args.country.country.clone(),
                            page: args.page,
                            pages: args.pages,
                            all: args.all,
                        },
                    )
                    .await?
                }
                Command::Chart(args) => {
                    commands::chart::run(
                        &client,
                        &ChartRequest {
                            chart: match args.chart {
                                ChartType::Top => appstore_api::requests::ChartType::Top,
                                ChartType::Free => appstore_api::requests::ChartType::Free,
                                ChartType::Paid => appstore_api::requests::ChartType::Paid,
                                ChartType::Grossing => appstore_api::requests::ChartType::Grossing,
                            },
                            country: args.country.country.clone(),
                            limit: args.limit,
                            category: args.category,
                        },
                    )
                    .await?
                }
                Command::InstallSkill | Command::List(_) | Command::App(_) => {
                    unreachable!("handled before creating the App Store client")
                }
            };
            serde_json::to_value(envelope)?
        }
    };
    output::emit(&value, cli.pretty, cli.output_file.as_deref())
}

fn client_config(cli: &Cli) -> ClientConfig {
    ClientConfig {
        timeout: Duration::from_secs(cli.timeout),
        retries: cli.retries,
    }
}

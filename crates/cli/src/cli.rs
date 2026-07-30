use std::{net::IpAddr, path::PathBuf};

use appstore_api::app_store::AppSpecifier;
use clap::{ArgGroup, Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "asapi",
    version,
    about = "Query Apple App Store data and run a local research workspace"
)]
pub struct Cli {
    /// Format JSON with indentation.
    #[arg(long, global = true)]
    pub pretty: bool,

    /// Save JSON to a file instead of printing it.
    #[arg(long, global = true)]
    pub output_file: Option<PathBuf>,

    /// Stop an Apple request after this many seconds.
    #[arg(long, global = true, default_value_t = 30, value_parser = clap::value_parser!(u64).range(1..=300))]
    pub timeout: u64,

    /// Retry transient Apple failures up to this many times.
    #[arg(long, global = true, default_value_t = 2, value_parser = clap::value_parser!(u8).range(0..=10))]
    pub retries: u8,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Install the app-store-research skill for an AI agent.
    InstallSkill,
    /// Search for apps by name or keyword.
    Search(SearchArgs),
    /// Get details for one or more apps.
    Lookup(LookupArgs),
    /// Compare an app's public ratings across multiple countries.
    Popularity(PopularityArgs),
    /// Get recent customer reviews for an app.
    Reviews(ReviewsArgs),
    /// Get the current top apps for a country or category.
    Chart(ChartArgs),
    /// List available countries, categories, or chart types.
    List(ListArgs),
    /// Run the local project application.
    App(AppArgs),
}

#[derive(Debug, Args)]
pub struct AppArgs {
    #[command(subcommand)]
    pub command: AppCommand,
}

#[derive(Debug, Subcommand)]
pub enum AppCommand {
    /// Start the local API and web interface.
    Serve(ServeArgs),
}

#[derive(Debug, Args)]
pub struct ServeArgs {
    /// Address to bind. The default is accessible only from this computer.
    #[arg(long, default_value = "127.0.0.1")]
    pub host: IpAddr,

    /// HTTP port.
    #[arg(long, default_value_t = 3000)]
    pub port: u16,

    /// Storage base. Databases are placed in <path>/asapi-storage/projects.
    #[arg(long)]
    pub storage_path: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct CountryArgs {
    /// App Store country code (overrides URL; default: URL storefront or us).
    #[arg(long)]
    pub country: Option<String>,
}

#[derive(Debug, Args)]
pub struct SearchArgs {
    pub term: String,
    #[command(flatten)]
    pub country: CountryArgs,
    #[arg(long, default_value_t = 10, value_parser = clap::value_parser!(u32).range(1..=200))]
    pub limit: u32,
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..=200))]
    pub local_limit: Option<u32>,
}

#[derive(Debug, Args)]
pub struct LookupArgs {
    #[arg(required = true, num_args = 1..=10)]
    pub apps: Vec<AppSpecifier>,
    #[command(flatten)]
    pub country: CountryArgs,
    /// Also scrape screenshots, in-app purchases, and similar apps.
    #[arg(long)]
    pub full: bool,
}

#[derive(Debug, Args)]
pub struct PopularityArgs {
    pub app: AppSpecifier,
    #[arg(long, value_enum, default_value_t = PopularityGroup::Tier1)]
    pub group: PopularityGroup,
    #[arg(long, value_delimiter = ',', num_args = 1..)]
    pub countries: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum PopularityGroup {
    Tier1,
    Tier2,
}

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("page_selection").args(["page", "pages", "all"]).multiple(false)))]
pub struct ReviewsArgs {
    pub app: AppSpecifier,
    #[command(flatten)]
    pub country: CountryArgs,
    #[arg(long, value_parser = clap::value_parser!(u8).range(1..=10))]
    pub page: Option<u8>,
    #[arg(long, value_parser = clap::value_parser!(u8).range(1..=10))]
    pub pages: Option<u8>,
    #[arg(long)]
    pub all: bool,
}

#[derive(Debug, Args)]
pub struct ChartArgs {
    #[arg(value_enum)]
    pub chart: ChartType,
    #[command(flatten)]
    pub country: CountryArgs,
    #[arg(long, default_value_t = 10, value_parser = clap::value_parser!(u16).range(1..=200))]
    pub limit: u16,
    #[arg(long)]
    pub category: Option<u32>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ChartType {
    Top,
    Free,
    Paid,
    Grossing,
}

#[derive(Debug, Args)]
pub struct ListArgs {
    #[arg(value_enum)]
    pub resource: ListResource,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ListResource {
    Countries,
    Categories,
    ChartTypes,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_has_only_serve() {
        let cli = Cli::try_parse_from(["asapi", "app", "serve"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::App(AppArgs {
                command: AppCommand::Serve(_)
            })
        ));
        assert!(Cli::try_parse_from(["asapi", "app", "project", "list"]).is_err());
    }

    #[test]
    fn existing_commands_keep_their_shape() {
        assert!(Cli::try_parse_from(["asapi", "lookup", "1", "2"]).is_ok());
        assert!(Cli::try_parse_from(["asapi", "lookup", "1", "--full"]).is_ok());
        assert!(Cli::try_parse_from(["asapi", "iap", "1"]).is_err());
        assert!(Cli::try_parse_from(["asapi", "similar", "1"]).is_err());
        assert!(Cli::try_parse_from(["asapi", "reviews", "1", "--all"]).is_ok());
        assert!(Cli::try_parse_from(["asapi", "chart", "free", "--limit", "0"]).is_err());
    }
}

use std::path::PathBuf;

use clap::{ArgGroup, Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "asapi",
    version,
    about = "Query Apple App Store apps, ratings, purchases, reviews, and charts"
)]
pub struct Cli {
    /// Format JSON with indentation.
    #[arg(long, global = true)]
    pub pretty: bool,

    /// Save JSON to a file instead of printing it.
    #[arg(long, global = true)]
    pub output_file: Option<PathBuf>,

    /// Stop a request after this many seconds.
    #[arg(long, global = true, default_value_t = 30, value_parser = clap::value_parser!(u64).range(1..=300))]
    pub timeout: u64,

    /// Retry transient failures up to this many times.
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
    /// Get in-app purchases displayed for an app.
    Iap(IapArgs),
    /// Get recent customer reviews for an app.
    Reviews(ReviewsArgs),
    /// Get the current top apps for a country or category.
    Chart(ChartArgs),
    /// List available countries, categories, or chart types.
    List(ListArgs),
}

#[derive(Debug, Args)]
pub struct CountryArgs {
    /// Two-letter App Store country code.
    #[arg(long, default_value = "us")]
    pub country: String,
}

#[derive(Debug, Args)]
pub struct SearchArgs {
    /// Search term.
    pub term: String,

    #[command(flatten)]
    pub country: CountryArgs,

    /// Maximum number of results requested from Apple (1-200).
    #[arg(long, default_value_t = 10, value_parser = clap::value_parser!(u32).range(1..=200))]
    pub limit: u32,

    /// Maximum number of results emitted locally (1-200; must not exceed --limit).
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..=200))]
    pub local_limit: Option<u32>,
}

#[derive(Debug, Args)]
pub struct LookupArgs {
    /// App Store IDs to look up (maximum 200).
    #[arg(required = true, num_args = 1..)]
    pub ids: Vec<u64>,

    #[command(flatten)]
    pub country: CountryArgs,
}

#[derive(Debug, Args)]
pub struct PopularityArgs {
    /// App Store ID.
    pub id: u64,

    /// Country group to query when --countries is not provided.
    #[arg(
        long,
        value_enum,
        default_value_t = PopularityGroup::Tier1,
        long_help = "Country group to query when --countries is not provided.\n\nTier 1: us, ca, cn, jp, gb, de, fr, kr, au.\nTier 2: all Tier 1 countries plus in, br, mx, es, it, nl, id, sg, hk, tw, ae."
    )]
    pub group: PopularityGroup,

    /// Comma-separated country codes; overrides --group.
    #[arg(long, value_delimiter = ',', num_args = 1..)]
    pub countries: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum PopularityGroup {
    Tier1,
    Tier2,
}

impl PopularityGroup {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tier1 => "tier1",
            Self::Tier2 => "tier2",
        }
    }
}

#[derive(Debug, Args)]
pub struct IapArgs {
    /// App Store ID.
    pub id: u64,

    #[command(flatten)]
    pub country: CountryArgs,
}

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("page_selection").args(["page", "pages", "all"]).multiple(false)))]
pub struct ReviewsArgs {
    /// App Store ID.
    pub id: u64,

    #[command(flatten)]
    pub country: CountryArgs,

    /// Get one page of reviews (1-10).
    #[arg(long, value_parser = clap::value_parser!(u8).range(1..=10))]
    pub page: Option<u8>,

    /// Get review pages 1 through N (1-10).
    #[arg(long, value_parser = clap::value_parser!(u8).range(1..=10))]
    pub pages: Option<u8>,

    /// Get all available review pages.
    #[arg(long)]
    pub all: bool,
}

impl ReviewsArgs {
    pub fn requested_pages(&self) -> Vec<u8> {
        if let Some(page) = self.page {
            vec![page]
        } else {
            (1..=self.pages.unwrap_or(if self.all { 10 } else { 1 })).collect()
        }
    }
}

#[derive(Debug, Args)]
pub struct ChartArgs {
    #[arg(value_enum)]
    pub chart: ChartType,

    #[command(flatten)]
    pub country: CountryArgs,

    /// Maximum number of apps (1-200).
    #[arg(long, default_value_t = 10, value_parser = clap::value_parser!(u16).range(1..=200))]
    pub limit: u16,

    /// App Store category ID.
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

impl ChartType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Free => "free",
            Self::Paid => "paid",
            Self::Grossing => "grossing",
        }
    }
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
    use clap::Parser;

    use super::*;

    #[test]
    fn review_page_modes_are_exclusive() {
        assert!(Cli::try_parse_from(["asapi", "reviews", "1", "--page", "2", "--all"]).is_err());
    }

    #[test]
    fn review_all_requests_apple_maximum() {
        let cli = Cli::try_parse_from(["asapi", "reviews", "1", "--all"]).unwrap();
        let Command::Reviews(args) = cli.command else {
            panic!("wrong command")
        };
        assert_eq!(args.requested_pages(), (1..=10).collect::<Vec<_>>());
    }

    #[test]
    fn invalid_limits_fail_during_argument_parsing() {
        assert!(Cli::try_parse_from(["asapi", "search", "test", "--limit", "201"]).is_err());
        assert!(Cli::try_parse_from(["asapi", "search", "test", "--local-limit", "0"]).is_err());
        assert!(Cli::try_parse_from(["asapi", "chart", "free", "--limit", "0"]).is_err());
    }

    #[test]
    fn parses_search_local_limit() {
        let cli = Cli::try_parse_from([
            "asapi",
            "search",
            "calendar",
            "--limit",
            "12",
            "--local-limit",
            "10",
        ])
        .unwrap();
        let Command::Search(args) = cli.command else {
            panic!("wrong command")
        };
        assert_eq!(args.limit, 12);
        assert_eq!(args.local_limit, Some(10));
    }

    #[test]
    fn parses_iap_country() {
        let cli = Cli::try_parse_from(["asapi", "iap", "42", "--country", "ae"]).unwrap();
        let Command::Iap(args) = cli.command else {
            panic!("wrong command")
        };
        assert_eq!(args.id, 42);
        assert_eq!(args.country.country, "ae");
    }

    #[test]
    fn popularity_defaults_to_tier1() {
        let cli = Cli::try_parse_from(["asapi", "popularity", "42"]).unwrap();
        let Command::Popularity(args) = cli.command else {
            panic!("wrong command")
        };
        assert_eq!(args.id, 42);
        assert_eq!(args.group, PopularityGroup::Tier1);
        assert_eq!(args.countries, None);
    }

    #[test]
    fn popularity_parses_country_override_and_group() {
        let cli = Cli::try_parse_from([
            "asapi",
            "popularity",
            "42",
            "--group",
            "tier2",
            "--countries",
            "jp,us,gb",
        ])
        .unwrap();
        let Command::Popularity(args) = cli.command else {
            panic!("wrong command")
        };
        assert_eq!(args.group, PopularityGroup::Tier2);
        assert_eq!(
            args.countries,
            Some(vec!["jp".into(), "us".into(), "gb".into()])
        );
    }

    #[test]
    fn parses_install_skill_without_options() {
        let cli = Cli::try_parse_from(["asapi", "install-skill"]).unwrap();
        assert!(matches!(cli.command, Command::InstallSkill));
    }
}

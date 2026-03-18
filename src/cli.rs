use anyhow::{bail, Result};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "dnsperf",
    version,
    about = "DNS Resolver Performance Tester\n\nTests the response time of various DNS resolvers by querying a set of\npopular domains and reporting average, minimum, and maximum latency.\n\nBy default, tests your ISP's DNS plus 9 well-known public resolvers.\nAdditional resolvers can be specified as positional arguments in the\nformat NAME:IP (e.g., \"MyDNS:10.0.0.1\")."
)]
pub struct Cli {
    /// Queries per domain per resolver
    #[arg(short, long, default_value_t = 3)]
    pub runs: u32,

    /// File with one domain per line (overrides built-in list)
    #[arg(short, long)]
    pub domains: Option<String>,

    /// Query timeout in seconds
    #[arg(short, long, default_value_t = 2)]
    pub timeout: u64,

    /// Suppress progress output; only show results
    #[arg(short, long)]
    pub quiet: bool,

    /// Disable colored output
    #[arg(long)]
    pub no_color: bool,

    /// Skip testing the ISP's default resolver
    #[arg(long)]
    pub no_isp: bool,

    /// Output results in CSV format instead of table
    #[arg(long)]
    pub csv: bool,

    /// Custom resolvers in NAME:IP format
    #[arg(value_name = "NAME:IP")]
    pub resolvers: Vec<String>,
}

impl Cli {
    pub fn parse_and_validate() -> Result<Self> {
        let cli = Self::parse();

        if cli.runs == 0 {
            bail!("--runs must be a positive integer");
        }
        if cli.timeout == 0 {
            bail!("--timeout must be a positive integer");
        }
        for r in &cli.resolvers {
            if !r.contains(':') {
                bail!(
                    "Invalid resolver format '{}'. Expected NAME:IP (e.g., MyDNS:10.0.0.1)",
                    r
                );
            }
        }
        if let Some(ref path) = cli.domains {
            if !std::path::Path::new(path).is_file() {
                bail!("Cannot read domains file: {}", path);
            }
        }

        Ok(cli)
    }
}

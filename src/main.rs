mod cli;
mod isp;
mod output;
mod query;
mod resolver;
mod stats;

use anyhow::Result;
use colored::Colorize;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse_and_validate()?;

    if cli.no_color
        || std::env::var("NO_COLOR").is_ok()
        || !std::io::IsTerminal::is_terminal(&std::io::stdout())
    {
        colored::control::set_override(false);
    }

    let domains = if let Some(ref path) = cli.domains {
        resolver::load_domains(path)?
    } else {
        resolver::default_domains()
    };

    let mut resolvers: Vec<resolver::Resolver> = Vec::new();

    if !cli.no_isp {
        match isp::detect_isp_dns() {
            Some(ip) => {
                resolvers.push(resolver::Resolver {
                    name: "ISP".into(),
                    ip,
                });
            }
            None => {
                eprintln!("{} Could not detect ISP DNS. Skipping.", "[WARN]".yellow());
            }
        }
    }

    resolvers.extend(resolver::builtin_resolvers());

    for entry in &cli.resolvers {
        let (name, ip) = resolver::parse_custom(entry)?;
        resolvers.push(resolver::Resolver { name, ip });
    }

    eprintln!(
        "{} Testing {} resolvers against {} domains ({} runs each)",
        "[INFO]".cyan(),
        resolvers.len(),
        domains.len(),
        cli.runs
    );
    eprintln!(
        "{} Total queries per resolver: {}",
        "[INFO]".cyan(),
        domains.len() * cli.runs as usize
    );
    if cli.warmup > 0 {
        eprintln!(
            "{} Warmup queries per resolver: {}",
            "[INFO]".cyan(),
            domains.len() * cli.warmup as usize
        );
    }
    eprintln!();

    let domains = Arc::new(domains);

    let mut tasks = tokio::task::JoinSet::new();
    for r in &resolvers {
        let name = r.name.clone();
        let ip = r.ip.clone();
        let domains = Arc::clone(&domains);
        let runs = cli.runs;
        let warmup = cli.warmup;
        let timeout = cli.timeout;
        if !cli.quiet {
            eprintln!("  Testing {} ({})", name, ip);
        }
        tasks.spawn(
            async move { query::test_resolver(name, ip, domains, runs, warmup, timeout).await },
        );
    }

    let mut results = Vec::new();
    while let Some(result) = tasks.join_next().await {
        let result = result?;
        if !cli.quiet {
            eprintln!(
                "  Finished {} ({}/{})",
                result.name,
                result.latencies.len(),
                result.total_queries
            );
        }
        results.push(result);
    }

    let mut all_stats: Vec<stats::ResolverStats> = results
        .iter()
        .map(stats::ResolverStats::from_result)
        .collect();
    stats::sort_stats(&mut all_stats);

    if cli.csv {
        output::render_csv(&all_stats);
    } else {
        output::render_table(&all_stats, domains.len(), cli.runs, cli.warmup, cli.timeout);
    }

    Ok(())
}

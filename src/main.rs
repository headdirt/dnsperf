mod cli;
mod isp;
mod output;
mod query;
mod resolver;
mod stats;

use anyhow::Result;
use colored::Colorize;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse_and_validate()?;

    // Setup colors
    if cli.no_color || std::env::var("NO_COLOR").is_ok() || !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        colored::control::set_override(false);
    }

    // Load domains
    let domains = if let Some(ref path) = cli.domains {
        resolver::load_domains(path)?
    } else {
        resolver::default_domains()
    };

    // Build resolver list
    let mut resolvers: Vec<resolver::Resolver> = Vec::new();

    // ISP resolver
    if !cli.no_isp {
        match isp::detect_isp_dns() {
            Some(ip) => {
                resolvers.push(resolver::Resolver {
                    name: format!("ISP ({})", ip),
                    ip: "default".into(),
                });
            }
            None => {
                eprintln!("{} Could not detect ISP DNS. Skipping.", "[WARN]".yellow());
            }
        }
    }

    // Built-in resolvers
    resolvers.extend(resolver::builtin_resolvers());

    // Custom resolvers
    for entry in &cli.resolvers {
        let (name, ip) = resolver::parse_custom(entry);
        resolvers.push(resolver::Resolver { name, ip });
    }

    // Print info
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
    eprintln!();

    // Spawn async tasks per resolver
    let mut handles = Vec::new();
    for r in &resolvers {
        let name = r.name.clone();
        let ip = r.ip.clone();
        let domains = domains.clone();
        let runs = cli.runs;
        let timeout = cli.timeout;
        let quiet = cli.quiet;
        handles.push(tokio::spawn(async move {
            query::test_resolver(name, ip, domains, runs, timeout, quiet).await
        }));
    }

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await?);
    }

    // Clear progress line
    if !cli.quiet {
        eprint!("\r{:80}\r", "");
    }

    // Compute stats and sort
    let mut all_stats: Vec<stats::ResolverStats> =
        results.iter().map(stats::ResolverStats::from_result).collect();
    stats::sort_stats(&mut all_stats);

    // Render output
    if cli.csv {
        output::render_csv(&all_stats);
    } else {
        output::render_table(&all_stats, domains.len(), cli.runs, cli.timeout);
    }

    Ok(())
}

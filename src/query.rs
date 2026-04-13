use anyhow::Result;
use hickory_resolver::config::{NameServerConfig, ResolverConfig, ResolverOpts};
use hickory_resolver::name_server::TokioConnectionProvider;
use hickory_resolver::proto::xfer::Protocol;
use hickory_resolver::TokioResolver;
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};
use tokio::time::timeout;

pub struct ResolverResult {
    pub name: String,
    pub ip: String,
    pub latencies: Vec<f64>,
    pub total_queries: u32,
}

pub async fn test_resolver(
    name: String,
    ip: String,
    domains: Vec<String>,
    runs: u32,
    timeout_secs: u64,
    quiet: bool,
) -> ResolverResult {
    let resolver = if ip == "default" {
        create_system_resolver(timeout_secs)
    } else {
        create_resolver(&ip, timeout_secs)
    };

    let total = domains.len() as u32 * runs;

    let resolver = match resolver {
        Ok(r) => r,
        Err(_) => {
            return ResolverResult {
                name,
                ip,
                latencies: vec![],
                total_queries: total,
            };
        }
    };
    let mut latencies = Vec::new();
    let timeout_dur = Duration::from_secs(timeout_secs);

    for domain in &domains {
        if !quiet {
            eprint!("\r  {:<22}  {:<20}", name, domain);
        }
        for _run in 1..=runs {
            let start = Instant::now();
            let result = timeout(timeout_dur, resolver.lookup_ip(domain.as_str())).await;
            let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

            match result {
                Ok(Ok(_)) => {
                    latencies.push(elapsed_ms);
                }
                _ => {
                    // timeout or lookup error — treated as failure
                }
            }
        }
    }

    ResolverResult {
        name,
        ip,
        latencies,
        total_queries: total,
    }
}

fn create_resolver(ip: &str, timeout_secs: u64) -> Result<TokioResolver> {
    let addr = SocketAddr::new(ip.parse::<IpAddr>()?, 53);
    let ns = NameServerConfig::new(addr, Protocol::Udp);
    let mut config = ResolverConfig::new();
    config.add_name_server(ns);
    let mut opts = ResolverOpts::default();
    opts.timeout = Duration::from_secs(timeout_secs);
    opts.attempts = 1;
    opts.cache_size = 0;
    let resolver = TokioResolver::builder_with_config(config, TokioConnectionProvider::default())
        .with_options(opts)
        .build();
    Ok(resolver)
}

fn create_system_resolver(timeout_secs: u64) -> Result<TokioResolver> {
    let mut builder = TokioResolver::builder_tokio()?;
    builder.options_mut().timeout = Duration::from_secs(timeout_secs);
    builder.options_mut().attempts = 1;
    builder.options_mut().cache_size = 0;
    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::create_resolver;

    #[test]
    fn create_resolver_accepts_ipv6() {
        let resolver = create_resolver("2606:4700:4700::1111", 2).unwrap();
        assert_eq!(resolver.options().cache_size, 0);
    }

    #[test]
    fn create_resolver_disables_cache() {
        let resolver = create_resolver("1.1.1.1", 2).unwrap();
        assert_eq!(resolver.options().cache_size, 0);
    }
}

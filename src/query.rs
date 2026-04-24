use anyhow::Result;
use hickory_resolver::config::{NameServerConfig, ResolverConfig, ResolverOpts};
use hickory_resolver::name_server::TokioConnectionProvider;
use hickory_resolver::proto::xfer::Protocol;
use hickory_resolver::TokioResolver;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
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
    domains: Arc<Vec<String>>,
    runs: u32,
    warmup: u32,
    timeout_secs: u64,
) -> ResolverResult {
    let resolver = create_resolver(&ip, timeout_secs);
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
    let mut latencies = Vec::with_capacity(total as usize);
    let timeout_dur = Duration::from_secs(timeout_secs);

    for _ in 0..warmup {
        for domain in domains.iter() {
            let _ = run_lookup(&resolver, domain, timeout_dur).await;
        }
    }

    let mut queries = query_plan(domains.len(), runs);
    shuffle_queries(&mut queries, seed_for(&name, &ip));

    for domain_idx in queries {
        if let Some(elapsed_ms) = run_lookup(&resolver, &domains[domain_idx], timeout_dur).await {
            latencies.push(elapsed_ms);
        }
    }

    ResolverResult {
        name,
        ip,
        latencies,
        total_queries: total,
    }
}

async fn run_lookup(resolver: &TokioResolver, domain: &str, timeout_dur: Duration) -> Option<f64> {
    let start = Instant::now();
    let result = timeout(timeout_dur, resolver.lookup_ip(domain)).await;
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

    match result {
        Ok(Ok(_)) => Some(elapsed_ms),
        _ => None,
    }
}

fn query_plan(domain_count: usize, runs: u32) -> Vec<usize> {
    (0..domain_count)
        .flat_map(|d| std::iter::repeat_n(d, runs as usize))
        .collect()
}

fn shuffle_queries(queries: &mut [usize], mut seed: u64) {
    if queries.len() < 2 {
        return;
    }

    for i in (1..queries.len()).rev() {
        seed = next_seed(seed);
        let j = (seed as usize) % (i + 1);
        queries.swap(i, j);
    }
}

fn seed_for(name: &str, ip: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    name.hash(&mut hasher);
    ip.hash(&mut hasher);
    hasher.finish()
}

fn next_seed(seed: u64) -> u64 {
    seed.wrapping_mul(6364136223846793005).wrapping_add(1)
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

#[cfg(test)]
mod tests {
    use super::{create_resolver, query_plan, seed_for, shuffle_queries};

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

    #[test]
    fn query_plan_repeats_each_domain_runs_times() {
        assert_eq!(query_plan(2, 3), vec![0, 0, 0, 1, 1, 1]);
    }

    #[test]
    fn shuffle_queries_preserves_workload() {
        let mut queries = query_plan(4, 3);
        let mut original = queries.clone();

        shuffle_queries(&mut queries, seed_for("Cloudflare", "1.1.1.1"));

        assert_ne!(queries, original);
        queries.sort();
        original.sort();
        assert_eq!(queries, original);
    }
}

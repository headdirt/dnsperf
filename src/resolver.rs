use anyhow::{bail, Result};
use std::fs;

#[derive(Debug, Clone)]
pub struct Resolver {
    pub name: String,
    pub ip: String,
}

pub fn builtin_resolvers() -> Vec<Resolver> {
    vec![
        Resolver { name: "Cloudflare".into(), ip: "1.1.1.1".into() },
        Resolver { name: "Google".into(), ip: "8.8.8.8".into() },
        Resolver { name: "Quad9".into(), ip: "9.9.9.9".into() },
        Resolver { name: "OpenDNS".into(), ip: "208.67.222.222".into() },
        Resolver { name: "AdGuard".into(), ip: "94.140.14.14".into() },
        Resolver { name: "Comodo".into(), ip: "8.26.56.26".into() },
        Resolver { name: "Level3".into(), ip: "4.2.2.1".into() },
        Resolver { name: "NextDNS".into(), ip: "45.90.28.0".into() },
        Resolver { name: "CleanBrowsing".into(), ip: "185.228.168.9".into() },
    ]
}

pub fn default_domains() -> Vec<String> {
    vec![
        "google.com",
        "facebook.com",
        "twitter.com",
        "netflix.com",
        "amazon.com",
        "apple.com",
        "microsoft.com",
        "github.com",
        "reddit.com",
        "wikipedia.org",
        "youtube.com",
        "linkedin.com",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

pub fn load_domains(path: &str) -> Result<Vec<String>> {
    let contents = fs::read_to_string(path)?;
    let domains: Vec<String> = contents
        .lines()
        .map(|line| line.split('#').next().unwrap_or("").trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    if domains.is_empty() {
        bail!("No valid domains found in {}", path);
    }
    Ok(domains)
}

pub fn parse_custom(s: &str) -> (String, String) {
    let idx = s.find(':').unwrap();
    let name = s[..idx].to_string();
    let ip = s[idx + 1..].to_string();
    (name, ip)
}

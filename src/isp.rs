use std::process::Command;

pub fn detect_isp_dns() -> Option<String> {
    detect_platform().or_else(detect_resolv_conf)
}

fn detect_platform() -> Option<String> {
    if cfg!(target_os = "macos") {
        let output = Command::new("scutil").arg("--dns").output().ok()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("nameserver[0]") {
                return trimmed
                    .split_once(':')
                    .and_then(|(_, value)| first_ip(value));
            }
        }
        None
    } else if cfg!(target_os = "linux") {
        let output = Command::new("resolvectl").arg("status").output().ok()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("DNS Servers:") {
                return trimmed.strip_prefix("DNS Servers:").and_then(first_ip);
            }
        }
        None
    } else {
        None
    }
}

fn detect_resolv_conf() -> Option<String> {
    let contents = std::fs::read_to_string("/etc/resolv.conf").ok()?;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("nameserver") {
            return trimmed.split_whitespace().nth(1).and_then(first_ip);
        }
    }
    None
}

fn first_ip(value: &str) -> Option<String> {
    value
        .split_whitespace()
        .find(|candidate| candidate.parse::<std::net::IpAddr>().is_ok())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::first_ip;

    #[test]
    fn first_ip_accepts_ipv4() {
        assert_eq!(first_ip(" 192.168.1.1 "), Some("192.168.1.1".into()));
    }

    #[test]
    fn first_ip_accepts_ipv6() {
        assert_eq!(
            first_ip(" 2606:4700:4700::1111 "),
            Some("2606:4700:4700::1111".into())
        );
    }

    #[test]
    fn first_ip_picks_first_valid_address() {
        assert_eq!(
            first_ip("not-an-ip 1.1.1.1 8.8.8.8"),
            Some("1.1.1.1".into())
        );
    }
}

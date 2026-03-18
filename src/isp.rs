use std::process::Command;

pub fn detect_isp_dns() -> Option<String> {
    let ip = detect_platform().or_else(detect_resolv_conf);
    ip.filter(|s| !s.is_empty())
}

fn detect_platform() -> Option<String> {
    if cfg!(target_os = "macos") {
        let output = Command::new("scutil").arg("--dns").output().ok()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("nameserver[0]") {
                return trimmed.split(':').nth(1).map(|s| s.trim().to_string());
            }
        }
        None
    } else if cfg!(target_os = "linux") {
        let output = Command::new("resolvectl")
            .arg("status")
            .output()
            .ok()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("DNS Servers:") {
                return trimmed
                    .strip_prefix("DNS Servers:")
                    .map(|s| s.trim().to_string());
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
            return trimmed.split_whitespace().nth(1).map(|s| s.to_string());
        }
    }
    None
}

use crate::stats::ResolverStats;
use colored::Colorize;

fn fmt_stat(val: Option<f64>, precision: usize) -> String {
    match val {
        Some(v) => format!("{:.prec$}", v, prec = precision),
        None => "N/A".into(),
    }
}

fn truncate_display(display: String) -> String {
    let chars: Vec<char> = display.chars().collect();
    if chars.len() > 27 {
        format!("{}...", chars[..24].iter().collect::<String>())
    } else {
        display
    }
}

fn csv_field(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

pub fn render_table(stats: &[ResolverStats], domain_count: usize, runs: u32, timeout_secs: u64) {
    let fastest = stats.iter().find(|s| s.avg.is_some()).map(|s| &s.name);
    let slowest = stats
        .iter()
        .rev()
        .find(|s| s.avg.is_some())
        .map(|s| &s.name);
    // Don't color if fastest == slowest (only one resolver with results)
    let slowest = if slowest == fastest { None } else { slowest };

    println!();
    println!(
        "{:<28} {:>10} {:>10} {:>10} {:>12}",
        "Resolver".bold(),
        "Avg (ms)".bold(),
        "Min (ms)".bold(),
        "Max (ms)".bold(),
        "Successful".bold(),
    );
    println!(
        "{:<28} {:>10} {:>10} {:>10} {:>12}",
        "----------------------------", "----------", "----------", "----------", "------------"
    );

    for s in stats {
        let display = if s.ip == "default" {
            s.name.clone()
        } else {
            format!("{} ({})", s.name, s.ip)
        };
        let display = truncate_display(display);

        let avg_str = fmt_stat(s.avg, 2);
        let min_str = fmt_stat(s.min, 3);
        let max_str = fmt_stat(s.max, 3);
        let ratio = format!("{}/{}", s.successes, s.total);

        let line = format!(
            "{:<28} {:>10} {:>10} {:>10} {:>12}",
            display, avg_str, min_str, max_str, ratio
        );

        if s.avg.is_none() {
            println!("{}", line.red());
        } else if Some(&s.name) == fastest {
            println!("{}", line.green());
        } else if Some(&s.name) == slowest {
            println!("{}", line.red());
        } else {
            println!("{}", line);
        }
    }

    let total = domain_count * runs as usize;
    println!();
    println!(
        "{}",
        format!(
            "Queries per resolver: {} domains x {} runs = {} total",
            domain_count, runs, total
        )
        .dimmed()
    );
    println!(
        "{}",
        format!("Timeout: {}s per query", timeout_secs).dimmed()
    );
}

pub fn render_csv(stats: &[ResolverStats]) {
    println!("resolver,ip,avg_ms,min_ms,max_ms,successful_queries");
    for s in stats {
        let avg_str = fmt_stat(s.avg, 2);
        let min_str = fmt_stat(s.min, 3);
        let max_str = fmt_stat(s.max, 3);
        println!(
            "{},{},{},{},{},{}/{}",
            csv_field(&s.name),
            csv_field(&s.ip),
            avg_str,
            min_str,
            max_str,
            s.successes,
            s.total
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{csv_field, truncate_display};

    #[test]
    fn csv_field_quotes_commas_and_quotes() {
        assert_eq!(csv_field("Home, \"Lab\""), "\"Home, \"\"Lab\"\"\"");
    }

    #[test]
    fn csv_field_leaves_simple_values_unquoted() {
        assert_eq!(csv_field("Cloudflare"), "Cloudflare");
    }

    #[test]
    fn truncate_display_is_utf8_safe() {
        let display = "測試".repeat(20);
        let truncated = truncate_display(display);
        assert!(truncated.ends_with("..."));
        assert_eq!(truncated.trim_end_matches("...").chars().count(), 24);
    }
}

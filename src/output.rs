use crate::stats::ResolverStats;
use colored::Colorize;

fn fmt_stat(val: Option<f64>, precision: usize) -> String {
    match val {
        Some(v) => format!("{:.prec$}", v, prec = precision),
        None => "N/A".into(),
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
        "----------------------------",
        "----------",
        "----------",
        "----------",
        "------------"
    );

    for s in stats {
        let display = if s.ip == "default" {
            s.name.clone()
        } else {
            format!("{} ({})", s.name, s.ip)
        };
        let display = if display.len() > 27 {
            format!("{}...", &display[..24])
        } else {
            display
        };

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
            s.name, s.ip, avg_str, min_str, max_str, s.successes, s.total
        );
    }
}

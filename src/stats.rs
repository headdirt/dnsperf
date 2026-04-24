use crate::query::ResolverResult;

pub struct ResolverStats {
    pub name: String,
    pub ip: String,
    pub avg: Option<f64>,
    pub median: Option<f64>,
    pub p95: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub successes: usize,
    pub total: u32,
}

impl ResolverStats {
    pub fn from_result(result: &ResolverResult) -> Self {
        let mut stats = ResolverStats {
            name: result.name.clone(),
            ip: result.ip.clone(),
            avg: None,
            median: None,
            p95: None,
            min: None,
            max: None,
            successes: result.latencies.len(),
            total: result.total_queries,
        };

        if result.latencies.is_empty() {
            return stats;
        }

        let mut sorted = result.latencies.clone();
        sorted.sort_by(f64::total_cmp);
        let sum: f64 = sorted.iter().sum();

        stats.avg = Some(sum / sorted.len() as f64);
        stats.median = Some(percentile(&sorted, 50.0));
        stats.p95 = Some(percentile(&sorted, 95.0));
        stats.min = Some(sorted[0]);
        stats.max = Some(sorted[sorted.len() - 1]);
        stats
    }

    pub fn failure_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            1.0 - self.successes as f64 / self.total as f64
        }
    }
}

pub fn sort_stats(stats: &mut [ResolverStats]) {
    stats.sort_by(|a, b| {
        a.failure_rate()
            .total_cmp(&b.failure_rate())
            .then_with(|| compare_optional(a.p95, b.p95))
            .then_with(|| compare_optional(a.avg, b.avg))
    });
}

fn compare_optional(a: Option<f64>, b: Option<f64>) -> std::cmp::Ordering {
    match (a, b) {
        (Some(a), Some(b)) => a.total_cmp(&b),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

fn percentile(sorted: &[f64], percentile: f64) -> f64 {
    debug_assert!(!sorted.is_empty());

    if sorted.len() == 1 {
        return sorted[0];
    }

    let rank = (percentile / 100.0) * (sorted.len() - 1) as f64;
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;

    if lower == upper {
        sorted[lower]
    } else {
        let weight = rank - lower as f64;
        sorted[lower] + (sorted[upper] - sorted[lower]) * weight
    }
}

#[cfg(test)]
mod tests {
    use super::{percentile, ResolverStats};
    use crate::query::ResolverResult;

    #[test]
    fn from_result_calculates_percentiles_and_failure_rate() {
        let result = ResolverResult {
            name: "Test".into(),
            ip: "192.0.2.1".into(),
            latencies: vec![40.0, 10.0, 20.0, 30.0],
            total_queries: 5,
        };

        let stats = ResolverStats::from_result(&result);

        assert_eq!(stats.avg, Some(25.0));
        assert_eq!(stats.median, Some(25.0));
        assert_eq!(stats.p95, Some(38.5));
        assert_eq!(stats.failure_rate(), 0.19999999999999996);
    }

    #[test]
    fn percentile_interpolates() {
        assert_eq!(percentile(&[10.0, 20.0, 30.0, 40.0], 50.0), 25.0);
        assert_eq!(percentile(&[10.0, 20.0, 30.0, 40.0], 95.0), 38.5);
    }
}

use crate::query::ResolverResult;

pub struct ResolverStats {
    pub name: String,
    pub ip: String,
    pub avg: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub successes: usize,
    pub total: u32,
}

impl ResolverStats {
    pub fn from_result(result: &ResolverResult) -> Self {
        if result.latencies.is_empty() {
            return ResolverStats {
                name: result.name.clone(),
                ip: result.ip.clone(),
                avg: None,
                min: None,
                max: None,
                successes: 0,
                total: result.total_queries,
            };
        }

        let count = result.latencies.len();
        let (sum, min, max) = result.latencies.iter().fold(
            (0.0f64, f64::INFINITY, f64::NEG_INFINITY),
            |(s, mn, mx), &v| (s + v, mn.min(v), mx.max(v)),
        );
        let avg = sum / count as f64;

        ResolverStats {
            name: result.name.clone(),
            ip: result.ip.clone(),
            avg: Some(avg),
            min: Some(min),
            max: Some(max),
            successes: count,
            total: result.total_queries,
        }
    }
}

pub fn sort_stats(stats: &mut [ResolverStats]) {
    stats.sort_by(|a, b| match (&a.avg, &b.avg) {
        (Some(a_avg), Some(b_avg)) => a_avg.total_cmp(b_avg),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });
}

use metrics::TransportMetrics;

use crate::system::rule::TransportRule;

pub mod metrics;

/// Factors of the path which are referenced when the path is created.
#[derive(Debug, Clone, PartialEq)]
pub struct PathConstructionFactors {
    /// Metrics which are referenced when the path is created.
    pub metrics: TransportMetrics,

    /// Rules which are referenced when the path is created.
    pub rule: TransportRule,
}

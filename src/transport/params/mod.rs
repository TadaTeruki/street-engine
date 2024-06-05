use metrics::PathMetrics;
use numeric::{Group, Stage};
use rules::TransportRules;

pub mod evaluation;
pub mod metrics;
pub mod numeric;
pub mod rules;

/// Parameters of path to be extended.
#[derive(Debug, Clone, PartialEq)]
pub struct PathParams {
    pub group: Group,
    pub stage: Stage,
    pub rules_start: TransportRules,
    pub metrics: PathMetrics,
    pub evaluation: f64,
}

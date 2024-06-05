use crate::core::geometry::{angle::Angle, site::Site};

use super::params::{
    evaluation::PathEvaluationFactors, metrics::PathMetrics, numeric::Stage, rules::TransportRules,
};

/// Provider of transport rules.
pub trait TransportRulesProvider {
    fn get_rules(
        &self,
        site: &Site,
        angle: Angle,
        stage: Stage,
        metrics: &PathMetrics,
    ) -> Option<TransportRules>;
}

/// Provider of terrain elevation.
pub trait TerrainProvider {
    fn get_elevation(&self, site: &Site) -> Option<f64>;
}

/// Provider of evaluation of the path.
pub trait PathEvaluator {
    fn evaluate(&self, factors: PathEvaluationFactors) -> Option<f64>;
}

/// Provider of random f64 values.
///
/// The range of the value is the same as the range of `f64` (not constrained).
pub trait RandomF64Provider {
    fn gen_f64(&mut self) -> f64;
}

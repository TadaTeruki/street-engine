use crate::core::geometry::{angle::Angle, site::Site};

use super::params::{
    metrics::PathMetrics, numeric::Stage, prioritization::PathPrioritizationFactors,
    rules::TransportRules,
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
pub trait PathPrioritizator {
    fn evaluate(&self, factors: PathPrioritizationFactors) -> Option<f64>;
}

/// Provider of random f64 values.
///
/// The range of the value is the same as the range of `f64` (not constrained).
pub trait RandomF64Provider {
    fn gen_f64(&mut self) -> f64;
}

use crate::core::geometry::site::Site;

use super::rules::TransportRules;

/// Provider of transport rules.
///
/// Rules for constructing a path are determined by each site.
pub trait TransportRulesProvider {
    fn get_rules(&self, site: &Site) -> Option<TransportRules>;
}

/// Provider of random f64 values.
///
/// The range of the value is the same as the range of `f64`, not constrained.
pub trait RandomF64Provider {
    fn random_f64(&mut self) -> f64;
}

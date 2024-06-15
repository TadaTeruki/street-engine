use crate::core::geometry::site::Site;

use super::params::{
    numeric::Stage, prioritization::PathPrioritizationFactors,
    rules::GrowthRules,
};

/// Provider of transport rules.
pub trait GrowthRulesProvider {
    fn get_rules(
        &self,
        site: &Site,
        stage: Stage,
    ) -> Option<GrowthRules>;
}

/// Provider of transport rules that always returns the same rules.
///
/// This is used only for testing purposes.
pub(crate) struct SameRulesProvider {
    rules: GrowthRules,
}

impl SameRulesProvider {
    pub fn new(rules: GrowthRules) -> Self {
        Self { rules }
    }
}

impl GrowthRulesProvider for SameRulesProvider {
    fn get_rules(
        &self,
        _site: &Site,
        _stage: Stage,
    ) -> Option<GrowthRules> {
        Some(self.rules.clone())
    }
}

/// Provider of terrain elevation.
pub trait TerrainProvider {
    fn get_elevation(&self, site: &Site) -> Option<f64>;
}

/// Terrain provider that provides a flat surface.
///
/// This is used only for testing purposes.
pub(crate) struct SurfaceTerrain {
    elevation: f64,
}

impl SurfaceTerrain {
    pub fn new(elevation: f64) -> Self {
        Self {
            elevation: elevation,
        }
    }
}

impl TerrainProvider for SurfaceTerrain {
    fn get_elevation(&self, _site: &Site) -> Option<f64> {
        Some(self.elevation)
    }
}

/// Terrain provider that provides an elevation based on the nearest spot which has a predefined elevation.
///
/// This is used only for testing purposes.
pub(crate) struct VoronoiTerrain {
    spots: Vec<(Site, f64)>,
}

impl VoronoiTerrain {
    pub fn new(spots: Vec<(Site, f64)>) -> Self {
        Self { spots }
    }
}

impl TerrainProvider for VoronoiTerrain {
    fn get_elevation(&self, site: &Site) -> Option<f64> {
        self.spots
            .iter()
            .map(|(spot, elevation)| (spot.distance(site), elevation))
            .min_by(|(distance1, _), (distance2, _)| distance1.total_cmp(distance2))
            .map(|(_, elevation)| *elevation)
    }
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

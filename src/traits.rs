use crate::{
    core::geometry::site::Site,
    system::{
        node::{numeric::Stage, TransportNode},
        path::{factor::metrics::TransportMetrics, TransportPath},
        rule::TransportRule,
    },
    unit::Elevation,
};

/// Provider of transport rules.
pub trait TransportRuleProvider {
    fn get_rule(
        &self,
        site: &Site,
        stage: Stage,
        metrics: TransportMetrics,
    ) -> Option<TransportRule>;
}

/// Provider of transport rules that always returns the same rules.
///
/// This is used only for testing purposes.
pub(crate) struct MockSameRuleProvider {
    rule: TransportRule,
}

impl MockSameRuleProvider {
    pub fn new(rule: TransportRule) -> Self {
        Self { rule }
    }
}

impl TransportRuleProvider for MockSameRuleProvider {
    fn get_rule(
        &self,
        _site: &Site,
        _stage: Stage,
        _metrics: TransportMetrics,
    ) -> Option<TransportRule> {
        Some(self.rule.clone())
    }
}

pub enum TerrainType {
    Land,
    Water,
    Void,
}

impl TerrainType {
    pub fn is_land(&self) -> bool {
        matches!(self, TerrainType::Land)
    }
}

/// Provider of terrain.
pub trait TerrainProvider {
    fn get_terrain(&self, site: &Site) -> (TerrainType, Option<Elevation>);
}

/// Terrain provider that provides a flat surface.
///
/// This is used only for testing purposes.
pub(crate) struct MockSurfaceTerrain {
    elevation: Elevation,
}

impl MockSurfaceTerrain {
    pub fn new(elevation: Elevation) -> Self {
        Self {
            elevation: elevation,
        }
    }
}

impl TerrainProvider for MockSurfaceTerrain {
    fn get_terrain(&self, _site: &Site) -> (TerrainType, Option<Elevation>) {
        (TerrainType::Land, Some(self.elevation))
    }
}

/// Terrain provider that provides an elevation based on the nearest spot which has a predefined elevation.
///
/// This is used only for testing purposes.
pub(crate) struct MockVoronoiTerrain {
    spots: Vec<(Site, Elevation)>,
}

impl MockVoronoiTerrain {
    pub fn new(spots: Vec<(Site, Elevation)>) -> Self {
        Self { spots }
    }
}

impl TerrainProvider for MockVoronoiTerrain {
    fn get_terrain(&self, site: &Site) -> (TerrainType, Option<Elevation>) {
        self.spots
            .iter()
            .map(|(spot, elevation)| (spot.distance(site), elevation))
            .min_by(|(distance1, _), (distance2, _)| distance1.total_cmp(distance2))
            .map(|(_, elevation)| {
                if elevation.value() > 0.0 {
                    (TerrainType::Land, Some(*elevation))
                } else {
                    (TerrainType::Water, Some(*elevation))
                }
            })
            .unwrap_or((TerrainType::Void, None))
    }
}

/// Path prioritizator.
pub trait PathPrioritizator {
    /// Calculate the priority of the path from the start node and the expected path.
    fn prioritize(&self, start_node: &TransportNode, path_expected: TransportPath) -> Option<f64>;
}

/// Provider of random f64 values.
///
/// The range of the value is the same as the range of `f64` (not constrained).
pub trait RandomF64Provider {
    fn gen_f64(&mut self) -> f64;
}

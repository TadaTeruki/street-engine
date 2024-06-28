use crate::core::geometry::site::Site;

use super::numeric::Stage;

/// Factors for prioritizing the path.
pub struct PathPrioritizationFactors {
    /// The start site of the path.
    pub site_start: Site,
    /// The end site of the path.
    pub site_end: Site,
    /// The length of the path.
    pub path_length: f64,
    /// The stage of the path.
    pub stage: Stage,
    /// Whether the path is a bridge.
    pub is_bridge: bool,
}

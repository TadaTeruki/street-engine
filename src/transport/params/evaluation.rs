use crate::core::geometry::{angle::Angle, site::Site};

use super::numeric::Stage;

/// Factors for evaluating path.
pub struct PathEvaluationFactors {
    /// The start site of the path.
    pub site_start: Site,
    /// The end site of the path.
    pub site_end: Site,
    /// The angle of the path.
    pub angle: Angle,
    /// The length of the path.
    pub path_length: f64,
    /// The stage of the path.
    pub stage: Stage,
    /// Whether the path creates a bridge.
    pub creates_bridge: bool,
}

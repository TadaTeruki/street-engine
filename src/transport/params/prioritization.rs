use crate::core::geometry::{angle::Angle, site::Site};

use super::numeric::Stage;

/// Factors for calculating the priority of the stump to grow a path.
pub struct PathPrioritizationFactors {
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

impl Default for PathPrioritizationFactors {
    fn default() -> Self {
        Self {
            site_start: Site::new(0.0, 0.0),
            site_end: Site::new(0.0, 0.0),
            angle: Angle::new(0.0),
            path_length: 0.0,
            stage: Stage::from_num(0),
            creates_bridge: false,
        }
    }
}

impl PathPrioritizationFactors {
    /// Create a new path prioritization factors.
    pub fn new(
        site_start: Site,
        site_end: Site,
        angle: Angle,
        path_length: f64,
        stage: Stage,
        creates_bridge: bool,
    ) -> Self {
        Self {
            site_start,
            site_end,
            angle,
            path_length,
            stage,
            creates_bridge,
        }
    }

    /// Get the start site of the path.
    pub fn site_start(&self) -> Site {
        self.site_start
    }

    /// Get the end site of the path.
    pub fn site_end(&self) -> Site {
        self.site_end
    }

    /// Get the angle of the path.
    pub fn angle(&self) -> Angle {
        self.angle
    }

    /// Get the length of the path.
    pub fn path_length(&self) -> f64 {
        self.path_length
    }

    /// Get the stage of the path.
    pub fn stage(&self) -> Stage {
        self.stage
    }

    /// Get whether the path creates a bridge.
    pub fn creates_bridge(&self) -> bool {
        self.creates_bridge
    }
}

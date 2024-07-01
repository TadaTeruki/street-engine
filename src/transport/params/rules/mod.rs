use branch::BranchRules;
use bridge::BridgeRules;
use direction::PathDirectionRules;

pub mod branch;
pub mod bridge;
pub mod direction;

/// Rules to construct a path.
#[derive(Debug, Clone, PartialEq)]
pub struct TransportRules {
    /// Normal length of the path.
    pub path_normal_length: f64,
    /// Extra length of the path to search intersections.
    pub path_extra_length_for_intersection: f64,

    /// Maximum elevation difference of the path.
    ///
    /// To extend a path, the elevation difference (=slope) between the start and end of the path should be less than this value.
    ///
    /// To construct grade-separate paths, the elevation difference between the paths should be more than this value.
    pub path_slope_elevation_diff_limit: ElevationDiffLimit,

    /// Probability of branching. If 1.0, the path will always create branch.
    pub branch_rules: BranchRules,

    /// Rules to determine the direction of the path.
    pub path_direction_rules: PathDirectionRules,

    /// Rules to create bridges.
    pub bridge_rules: BridgeRules,
}

impl Default for TransportRules {
    fn default() -> Self {
        Self {
            path_normal_length: 0.0,
            path_extra_length_for_intersection: 0.0,
            path_slope_elevation_diff_limit: ElevationDiffLimit::AlwaysAllow,
            branch_rules: BranchRules::default(),
            path_direction_rules: PathDirectionRules::default(),
            bridge_rules: BridgeRules::default(),
        }
    }
}

impl TransportRules {
    /// Set the normal length of the path.
    pub fn path_normal_length(mut self, path_normal_length: f64) -> Self {
        self.path_normal_length = path_normal_length;
        self
    }

    /// Set the extra length of the path to search intersections.
    pub fn path_extra_length_for_intersection(
        mut self,
        path_extra_length_for_intersection: f64,
    ) -> Self {
        self.path_extra_length_for_intersection = path_extra_length_for_intersection;
        self
    }

    /// Set the maximum elevation difference of the path.
    pub fn path_slope_elevation_diff_limit(
        mut self,
        path_slope_elevation_diff_limit: ElevationDiffLimit,
    ) -> Self {
        self.path_slope_elevation_diff_limit = path_slope_elevation_diff_limit;
        self
    }

    /// Set the probability of branching.
    pub fn branch_rules(mut self, branch_rules: BranchRules) -> Self {
        self.branch_rules = branch_rules;
        self
    }

    /// Set the rules to determine the direction of the path.
    pub fn path_direction_rules(mut self, path_direction_rules: PathDirectionRules) -> Self {
        self.path_direction_rules = path_direction_rules;
        self
    }

    /// Set the rules to create bridges.
    pub fn bridge_rules(mut self, bridge_rules: BridgeRules) -> Self {
        self.bridge_rules = bridge_rules;
        self
    }
}

/// The limit of the elevation difference.
#[derive(Debug, Clone, PartialEq)]
pub enum ElevationDiffLimit {
    /// Always allow to construct a path.
    AlwaysAllow,
    /// Always deny to construct a path.
    AlwaysDeny,
    /// The limit will be proportional to the path length. (specified elevation * path length)
    Linear(f64),
    /// The limit will be a non-linear function of the path length.
    NonLinear(fn(f64) -> f64),
}

impl ElevationDiffLimit {
    /// Get the elevation difference from the path length.
    fn value(&self, path_length: f64) -> f64 {
        match self {
            ElevationDiffLimit::AlwaysAllow => f64::INFINITY,
            ElevationDiffLimit::AlwaysDeny => f64::NEG_INFINITY,
            ElevationDiffLimit::Linear(elevation) => elevation * path_length,
            ElevationDiffLimit::NonLinear(f) => f(path_length),
        }
    }

    /// Check if the slope is proper to construct a path.
    pub fn check_slope(&self, elevations: (f64, f64), path_length: f64) -> bool {
        let elevation_diff = (elevations.1 - elevations.0).abs();
        elevation_diff <= self.value(path_length)
    }
}

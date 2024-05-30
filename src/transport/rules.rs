/// Rules to construct a path.
#[derive(Debug, Clone, PartialEq)]
pub struct TransportRules {
    /// Priority to construct a path to this site.
    pub path_priority: f64,

    /// Elevation.
    pub elevation: f64,
    /// Population density.
    pub population_density: f64,

    /// Normal length of the path.
    pub path_normal_length: f64,
    /// Extra length of the path to search intersections.
    pub path_extra_length_for_intersection: f64,

    /// Maximum elevation difference of the path per 1.0 length.
    pub path_max_elevation_diff: Option<f64>,

    /// Probability of branching. If 1.0, the path will always create branch.
    pub branch_rules: BranchRules,

    /// Rules to determine the direction of the path.
    pub path_direction_rules: PathDirectionRules,

    /// Rules to create bridges.
    pub bridge_rules: BridgeRules,
}

/// Rules to create branches.
///
/// With `Default` values, the path will never create a branch.
#[derive(Debug, Clone, PartialEq)]
pub struct BranchRules {
    /// Density of intersections (probability of branching). If 1.0, the path will always create intersection.
    pub branch_density: f64,

    /// Probability of staging.
    pub staging_probability: f64,
}

impl Default for BranchRules {
    fn default() -> Self {
        Self {
            branch_density: 0.0,
            staging_probability: 0.0,
        }
    }
}

/// Rules to create bridges.
///
/// Bridge is the path that connect two distant sites where the normal path cannot be constructed.
/// For this package, the meaning of bridges includes not only the bridges over rivers or valleys but also tunnels under mountains.
///
/// With `Default` values, the path will never create a bridge.

#[derive(Debug, Clone, PartialEq)]
pub struct BridgeRules {
    /// Maximum length of bridges.
    pub max_bridge_length: f64,

    /// Number of comparison steps to create a bridge.
    /// When creating a bridge, the generator checks the nearest land site with no
    pub comparison_step: usize,
}

impl Default for BridgeRules {
    fn default() -> Self {
        Self {
            max_bridge_length: 0.0,
            comparison_step: 0,
        }
    }
}

/// Rules to determine the direction of a path.
///
/// With `Default` values, the path is always constructed as a straight line.
#[derive(Debug, Clone, PartialEq)]
pub struct PathDirectionRules {
    /// Maximum angle of curves.
    pub max_radian: f64,
    /// Number of candidates of the next site to create a path.
    /// This parameter should be an odd number to evaluate the straight path.
    pub comparison_step: usize,
}

impl Default for PathDirectionRules {
    fn default() -> Self {
        Self {
            max_radian: 0.0,
            comparison_step: 1,
        }
    }
}

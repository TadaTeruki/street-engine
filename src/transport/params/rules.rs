/// Rules to construct a path.
#[derive(Debug, Clone, PartialEq)]
pub struct GrowthRules {
    /// Normal length of the path.
    pub path_normal_length: f64,
    /// Extra length of the path to search intersections.
    pub path_extra_length_for_intersection: f64,

    /// Maximum elevation difference of the path.
    ///
    /// To extend a path, the elevation difference (=slope) between the start and end of the path should be less than this value.
    ///
    /// To construct grade-separate paths, the elevation difference between the paths should be more than this value.
    pub path_elevation_diff_limit: Option<f64>,

    /// Probability of branching. If 1.0, the path will always create branch.
    pub branch_rules: BranchRules,

    /// Rules to determine the direction of the path.
    pub path_direction_rules: PathDirectionRules,

    /// Rules to create bridges.
    pub bridge_rules: BridgeRules,
}

impl Default for GrowthRules {
    fn default() -> Self {
        Self {
            path_normal_length: 0.0,
            path_extra_length_for_intersection: 0.0,
            path_elevation_diff_limit: None,
            branch_rules: Default::default(),
            path_direction_rules: Default::default(),
            bridge_rules: Default::default(),
        }
    }
}

impl GrowthRules {
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
    pub fn path_elevation_diff_limit(mut self, path_elevation_diff_limit: Option<f64>) -> Self {
        self.path_elevation_diff_limit = path_elevation_diff_limit;
        self
    }

    /// Set the rules to create branches.
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

    /// Check the elevation difference which is allowed to create a path on land.
    pub fn check_elevation_diff_to_create_path_on_land(
        &self,
        elevation_start: f64,
        elevation_end: f64,
        distance: f64,
    ) -> bool {
        let allowed_elevation_diff =
            if let Some(max_elevation_diff) = self.path_elevation_diff_limit {
                max_elevation_diff
            } else {
                // always allowed
                return true;
            };
        let real_elevation_diff = (elevation_end - elevation_start).abs();

        real_elevation_diff <= allowed_elevation_diff * distance
    }
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

impl BranchRules {
    /// Set the density of intersections (probability of branching).
    pub fn branch_density(mut self, branch_density: f64) -> Self {
        self.branch_density = branch_density;
        self
    }

    /// Set the probability of staging.
    pub fn staging_probability(mut self, staging_probability: f64) -> Self {
        self.staging_probability = staging_probability;
        self
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

    /// Number of check steps to create a bridge.
    pub check_step: usize,
}

impl Default for BridgeRules {
    fn default() -> Self {
        Self {
            max_bridge_length: 0.0,
            check_step: 0,
        }
    }
}

impl BridgeRules {
    /// Set the maximum length of bridges.
    pub fn max_bridge_length(mut self, max_bridge_length: f64) -> Self {
        self.max_bridge_length = max_bridge_length;
        self
    }

    /// Set the number of check steps to create a bridge.
    pub fn check_step(mut self, check_step: usize) -> Self {
        self.check_step = check_step;
        self
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

impl PathDirectionRules {
    /// Set the maximum angle of curves.
    pub fn max_radian(mut self, max_radian: f64) -> Self {
        self.max_radian = max_radian;
        self
    }

    /// Set the number of candidates of the next site to create a path.
    pub fn comparison_step(mut self, comparison_step: usize) -> Self {
        self.comparison_step = comparison_step;
        self
    }
}

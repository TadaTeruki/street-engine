use crate::core::geometry::site::Site;

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

    /// Probability of branching. If 1.0, the path will always create branch.
    pub branch_probability: f64,

    /// Rules to determine the direction of the path.
    pub path_direction_rules: PathDirectionRules,
}

impl Default for TransportRules {
    /// With default values, the path is always constructed as a straight line.
    fn default() -> Self {
        Self {
            path_priority: 0.0,
            elevation: 0.0,
            population_density: 0.0,
            path_normal_length: 0.0,
            path_extra_length_for_intersection: 0.0,
            branch_probability: 0.0,
            path_direction_rules: PathDirectionRules::default(),
        }
    }
}

/// Rules to determine the direction of a path.
///
/// This struct implements `Default` method.
/// With default values, the path is always constructed as a straight line.
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

pub trait TransportRulesProvider {
    fn get_rules(&self, site: &Site) -> Option<TransportRules>;
}

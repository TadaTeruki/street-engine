use crate::core::geometry::site::Site;

/// Properties of a site for constructing a new path.
#[derive(Debug, Clone, PartialEq)]
pub struct TransportProperty {
    /// Priority to construct a path to this site.
    pub path_priority: f64,

    /// Elevation.
    pub elevation: f64,
    /// Population density.
    pub population_density: f64,

    /// Normal length of the path.
    pub path_normal_length: f64,
    /// Minimum length of the path.
    pub path_min_length: f64,

    /// Probability of branching. If 1.0, the path will always create branch.
    pub branch_probability: f64,

    /// Property of curves.
    /// If None, the path will be always extended to straight.
    pub curve: Option<CurveProperty>,
}

impl Default for TransportProperty {
    /// With default values, the path is always constructed as a straight line.
    fn default() -> Self {
        Self {
            path_priority: 0.0,
            elevation: 0.0,
            population_density: 0.0,
            path_normal_length: 0.0,
            path_min_length: 0.0,
            branch_probability: 0.0,
            curve: None,
        }
    }
}

/// Properties of curves.
#[derive(Debug, Clone, PartialEq)]
pub struct CurveProperty {
    /// Maximum angle of curves.
    pub max_radian: f64,
    /// Number of candidates of the next site to create a path.
    /// This parameter should be an odd number to evaluate the straight path.
    pub comparison_step: usize,
}

impl Default for CurveProperty {
    /// With default values, the path is always constructed as a straight line.
    fn default() -> Self {
        Self {
            max_radian: 0.0,
            comparison_step: 1,
        }
    }
}

pub trait TransportPropertyProvider {
    fn get_property(&self, site: &Site) -> Option<TransportProperty>;
}
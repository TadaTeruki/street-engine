use crate::core::geometry::site::Site;

/// Properties of a site for constructing a new path.
pub struct TransportProperty {
    /// Priority to construct a path to this site.
    pub path_priority: f64,

    /// Elevation.
    pub elevation: f64,
    /// Population density.
    pub population_density: f64,

    /// Probability of branching. If 1.0, the path will always create branch.
    pub branch_probability: f64,
    /// Length of the path.
    pub path_length: f64,

    /// Property of curves.
    /// If None, the path will be always extended to straight.
    pub curve: Option<CurveProperty>,
}

/// Properties of curves.
pub struct CurveProperty {
    /// Maximum angle of curves.
    pub max_radian: f64,
    /// Number of candidates of the next site to create a path.
    /// This parameter should be an odd number to evaluate the straight path.
    /// If the `max_curve_radian` is None, this parameter will be fixed to 1.
    pub comparison_step: usize,
}

impl TransportProperty {
    /// Create a new `TransportProperty`.
    pub fn new(
        path_priority: f64,
        elevation: f64,
        population_density: f64,
        branch_probability: f64,
        path_length: f64,
        curve: Option<CurveProperty>,
    ) -> Self {
        Self {
            path_priority,
            elevation,
            population_density,
            branch_probability,
            path_length,
            curve,
        }
    }
}

impl CurveProperty {
    /// Create a new `CurveProperty`.
    pub fn new(max_radian: f64, comparison_step: usize) -> Self {
        Self {
            max_radian,
            comparison_step,
        }
    }
}

pub trait TransportPropertyProvider {
    fn get_property(&self, site: &Site) -> TransportProperty;
}

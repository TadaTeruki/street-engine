/// Rules to determine the direction of a path.
///
/// With `Default` values, the path is always constructed as a straight line.
#[derive(Debug, Clone, PartialEq)]
pub struct PathDirectionRule {
    /// Maximum angle of curves.
    pub max_radian: f64,
    /// Number of candidates of the next site to create a path.
    /// This parameter should be an odd number to evaluate the straight path.
    pub comparison_step: usize,
}

impl Default for PathDirectionRule {
    fn default() -> Self {
        Self {
            max_radian: 0.0,
            comparison_step: 1,
        }
    }
}

impl PathDirectionRule {
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

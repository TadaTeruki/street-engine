/// Rules to create branches.
///
/// With `Default` values, the path will never create a branch.
#[derive(Debug, Clone, PartialEq)]
pub struct BranchRule {
    /// Density of junctions (=probability of branching). If 1.0, the path will always create a junction.
    pub branch_density: f64,

    /// Probability of staging.
    pub staging_probability: f64,
}

impl Default for BranchRule {
    fn default() -> Self {
        Self {
            branch_density: 0.0,
            staging_probability: 0.0,
        }
    }
}

impl BranchRule {
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

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

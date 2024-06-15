/*
/// Metrics for a path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PathMetrics {
    /// The number of times the path has been extended from origin node.
    pub extend_count: usize,
    /// The number of times the path has been extended from the last staged node.
    pub extend_count_since_last_staged: usize,
    /// The number of times the path has been extended from the last branched node.
    pub extend_count_since_last_branched: usize,
    /// The number of times the path has been branched.
    pub branch_count: usize,
}

impl PathMetrics {
    pub fn initial() -> Self {
        Self::default()
    }

    pub fn incremented(&self, staged: bool, branched: bool) -> Self {
        let count_last_staged = if staged {
            0
        } else {
            self.extend_count_since_last_staged + 1
        };

        let count_last_branched = if branched {
            0
        } else {
            self.extend_count_since_last_branched + 1
        };

        let branch_count = if branched {
            self.branch_count + 1
        } else {
            self.branch_count
        };

        Self {
            extend_count: self.extend_count + 1,
            extend_count_since_last_staged: count_last_staged,
            extend_count_since_last_branched: count_last_branched,
            branch_count,
        }
    }
}
*/
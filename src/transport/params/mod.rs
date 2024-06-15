use numeric::Stage;
use rules::TransportRules;

pub mod metrics;
pub mod numeric;
pub mod prioritization;
pub mod rules;

/// Parameters of stump.
#[derive(Debug, Clone, PartialEq)]
pub struct StumpParams {
    pub stage: Stage,
    pub rules: TransportRules,
    pub priority: f64,
}

impl Default for StumpParams {
    fn default() -> Self {
        Self {
            stage: Default::default(),
            rules: Default::default(),
            priority: 0.0,
        }
    }
}

impl StumpParams {
    /// Set the stage of the stump.
    pub fn stage(mut self, stage: Stage) -> Self {
        self.stage = stage;
        self
    }

    /// Set the rules of the stump.
    pub fn rules(mut self, rules_start: TransportRules) -> Self {
        self.rules = rules_start;
        self
    }

    /// Set the priority of the stump.
    pub fn priority(mut self, priority: f64) -> Self {
        self.priority = priority;
        self
    }
}

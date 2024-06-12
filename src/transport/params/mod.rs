use metrics::PathMetrics;
use numeric::Stage;
use rules::TransportRules;

pub mod evaluation;
pub mod metrics;
pub mod numeric;
pub mod rules;

/// Parameters of path to be extended.
#[derive(Debug, Clone, PartialEq)]
pub struct PathParams {
    pub stage: Stage,
    pub rules_start: TransportRules,
    pub metrics: PathMetrics,
    pub evaluation: f64,
}

impl Default for PathParams {
    fn default() -> Self {
        Self {
            stage: Default::default(),
            rules_start: Default::default(),
            metrics: Default::default(),
            evaluation: 0.0,
        }
    }
}

impl PathParams {
    /// Set the stage of the path.
    pub fn stage(mut self, stage: Stage) -> Self {
        self.stage = stage;
        self
    }

    /// Set the rules to start the path.
    pub fn rules_start(mut self, rules_start: TransportRules) -> Self {
        self.rules_start = rules_start;
        self
    }

    /// Set the metrics of the path.
    pub fn metrics(mut self, metrics: PathMetrics) -> Self {
        self.metrics = metrics;
        self
    }

    /// Set the evaluation of the path.
    pub fn evaluation(mut self, evaluation: f64) -> Self {
        self.evaluation = evaluation;
        self
    }
}

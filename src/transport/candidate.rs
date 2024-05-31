use crate::core::{container::path_network::NodeId, geometry::angle::Angle, Stage};

use super::{node::TransportNode, rules::TransportRules};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PathCandidate {
    pub node_start: TransportNode,
    pub node_start_id: NodeId,
    pub angle_expected_end: Angle,
    pub stage: Stage,
    pub priority: f64,
    pub rules_start: TransportRules,
}

impl Eq for PathCandidate {}

impl PartialOrd for PathCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.priority.partial_cmp(&other.priority)
    }
}

impl Ord for PathCandidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.total_cmp(&other.priority)
    }
}
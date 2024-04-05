use crate::core::{container::path_network::NodeId, geometry::angle::Angle};

use super::property::TransportProperty;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransportNode {
    pub node_id: NodeId,
}

impl TransportNode {
    pub fn new(node_id: NodeId) -> Self {
        Self { node_id }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PathCandidate {
    pub node_from: TransportNode,
    pub angle_to: Angle,
    pub property: TransportProperty,
}

impl PathCandidate {
    pub fn new(node_from: TransportNode, angle_to: Angle, property: TransportProperty) -> Self {
        Self {
            node_from,
            angle_to,
            property,
        }
    }
}

impl Eq for PathCandidate {}

impl PartialOrd for PathCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathCandidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.property
            .path_priority
            .total_cmp(&other.property.path_priority)
    }
}

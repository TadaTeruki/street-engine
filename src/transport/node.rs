use crate::core::geometry::{angle::Angle, line_segment::LineSegment, site::Site};

use super::property::TransportProperty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransportNode {
    pub site: Site,
}

impl TransportNode {
    pub fn new(site: Site) -> Self {
        Self { site }
    }
}

impl From<TransportNode> for Site {
    fn from(node: TransportNode) -> Self {
        node.site
    }
}

impl PartialOrd for TransportNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TransportNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.site.cmp(&other.site)
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

#[derive(Debug)]
enum NextTransportNodeType {
    New(TransportNode),
    Existing(TransportNode),
    Intersect(TransportNode, LineSegment),
}

impl NextTransportNodeType {
    fn node_to(&self) -> TransportNode {
        match self {
            Self::New(node) => *node,
            Self::Existing(node) => *node,
            Self::Intersect(node, _) => *node,
        }
    }
}

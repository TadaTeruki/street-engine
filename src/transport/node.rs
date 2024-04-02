use crate::core::geometry::{angle::Angle, site::Site};

use super::property::CurveProperty;

#[derive(Debug, Clone, Copy)]
pub struct TransportNode {
    pub site: Site,
    pub elevated_height: f64,
}

impl TransportNode {
    pub fn new(site: Site, elevated_height: f64) -> Self {
        Self {
            site,
            elevated_height,
        }
    }
}

impl From<TransportNode> for Site {
    fn from(node: TransportNode) -> Self {
        node.site
    }
}

impl PartialEq for TransportNode {
    fn eq(&self, other: &Self) -> bool {
        self.site == other.site && self.elevated_height == other.elevated_height
    }
}

impl Eq for TransportNode {}

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
    pub path_length: f64,
    pub path_priority: f64,
    pub curve: Option<CurveProperty>,
}

impl PathCandidate {
    pub fn new(
        node_from: TransportNode,
        angle_to: Angle,
        path_length: f64,
        path_priority: f64,
        curve: Option<CurveProperty>,
    ) -> Self {
        Self {
            node_from,
            angle_to,
            path_length,
            path_priority,
            curve,
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
        self.path_priority.total_cmp(&other.path_priority)
    }
}

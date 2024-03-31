use crate::core::geometry::{angle::Angle, site::Site};

#[derive(Debug, Clone, Copy, PartialOrd, Ord)]
pub struct TransportNode {
    site: Site,
}

impl TransportNode {
    pub fn new(site: Site) -> Self {
        Self { site }
    }
}

impl Into<Site> for TransportNode {
    fn into(self) -> Site {
        self.site
    }
}

impl PartialEq for TransportNode {
    fn eq(&self, other: &Self) -> bool {
        self.site == other.site
    }
}

impl Eq for TransportNode {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathCandidate {
    pub from: TransportNode,
    pub to_angle: Angle,
    pub path_priority: f64,
}

impl PathCandidate {
    pub fn new(from: TransportNode, to_angle: Angle, path_priority: f64) -> Self {
        Self {
            from,
            to_angle,
            path_priority,
        }
    }
}

impl Eq for PathCandidate {}

impl PartialOrd for PathCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.path_priority.partial_cmp(&other.path_priority)
    }
}

impl Ord for PathCandidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path_priority.total_cmp(&other.path_priority)
    }
}

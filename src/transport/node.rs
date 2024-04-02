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

impl Into<Site> for TransportNode {
    fn into(self) -> Site {
        self.site
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
        let ordering = self.site.partial_cmp(&other.site);
        if ordering == Some(std::cmp::Ordering::Equal) {
            self.elevated_height.partial_cmp(&other.elevated_height)
        } else {
            ordering
        }
    }
}

impl Ord for TransportNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let ordering = self.site.cmp(&other.site);
        if ordering == std::cmp::Ordering::Equal {
            self.elevated_height.total_cmp(&other.elevated_height)
        } else {
            ordering
        }
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
        self.path_priority.partial_cmp(&other.path_priority)
    }
}

impl Ord for PathCandidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path_priority.total_cmp(&other.path_priority)
    }
}

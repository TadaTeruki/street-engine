use crate::{core::geometry::site::Site, transport::params::numeric::Stage};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct TransportNode {
    pub site: Site,
    pub elevation: f64,
    pub(crate) stage: Stage,
    pub(crate) is_bridge: bool,
}

impl TransportNode {
    pub fn new(site: Site, elevation: f64, stage: Stage, is_bridge: bool) -> Self {
        Self {
            site,
            elevation,
            stage,
            is_bridge,
        }
    }

    pub fn path_is_bridge(&self, other: &Self) -> bool {
        self.is_bridge || other.is_bridge
    }

    pub fn path_stage(&self, other: &Self) -> Stage {
        self.stage.max(other.stage)
    }

    pub fn elevation_on_path(&self, other: &Self, site: Site) -> f64 {
        let distance_0 = self.site.distance(&site);
        let distance_1 = other.site.distance(&site);
        let prop_start = distance_1 / (distance_0 + distance_1);
        self.elevation * prop_start + other.elevation * (1.0 - prop_start)
    }
}

impl Eq for TransportNode {}

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

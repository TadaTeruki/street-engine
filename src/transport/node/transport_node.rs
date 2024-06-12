use crate::{core::geometry::site::Site, transport::params::numeric::Stage};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct TransportNode {
    pub site: Site,
    pub(crate) stage: Stage,
}

impl TransportNode {
    pub fn new(site: Site, stage: Stage) -> Self {
        Self {
            site,
            stage,
        }
    }
    
    pub fn path_stage(&self, other: &Self) -> Stage {
        self.stage.max(other.stage)
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

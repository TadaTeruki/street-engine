use crate::core::{geometry::site::Site, Stage};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TransportNode {
    pub site: Site,
    pub stage: Stage,
    pub is_bridge: bool,
}

impl TransportNode {
    pub fn new(site: Site, stage: Stage, is_bridge: bool) -> Self {
        Self {
            site,
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

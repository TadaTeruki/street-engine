use nodetype::TransportNodeType;
use numeric::Stage;

use crate::core::{container::path_network::PathNetworkNodeTrait, geometry::site::Site};

pub mod nodetype;
pub mod numeric;

/// A node in the transport network.
#[derive(Debug, Clone, PartialEq)]
pub struct TransportNode {
    /// Site of the node.
    site: Site,

    /// Stage of the node.
    stage: Stage,

    /// Type of the node (e.g. land, bridge, tunnel, etc.)
    nodetype: TransportNodeType,
}

impl TransportNode {
    pub fn new(site: Site, stage: Stage, nodetype: TransportNodeType) -> Self {
        Self {
            site,
            stage,
            nodetype,
        }
    }

    pub fn get_site(&self) -> Site {
        self.site
    }

    pub fn get_stage(&self) -> Stage {
        self.stage
    }

    pub fn get_nodetype(&self) -> TransportNodeType {
        self.nodetype
    }
}

impl TransportNode {
    pub fn path_stage(&self, other: &Self) -> Stage {
        self.stage.max(other.stage)
    }
}

impl Eq for TransportNode {}

impl PathNetworkNodeTrait for TransportNode {
    fn get_site(&self) -> Site {
        self.site
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

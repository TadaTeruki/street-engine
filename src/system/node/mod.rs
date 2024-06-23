use nodetype::TransportNodeType;
use numeric::Stage;

use crate::{
    core::{container::path_network::PathNetworkNodeTrait, geometry::site::Site},
    unit::Length,
};

pub mod nodetype;
pub mod numeric;

/// A node in the transport network.
#[derive(Debug, Clone, PartialEq)]
pub struct TransportNode {
    /// Site of the node.
    site: Site,

    /// Stage of the node.
    stage: Stage,

    /// Radius of the node.
    radius: Length,

    /// Type of the node (e.g. land, bridge, tunnel, etc.)
    nodetype: TransportNodeType,
}

impl TransportNode {
    pub fn new(site: Site, stage: Stage, radius: Length, nodetype: TransportNodeType) -> Self {
        Self {
            site,
            stage,
            radius,
            nodetype,
        }
    }

    pub fn get_site(&self) -> Site {
        self.site
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

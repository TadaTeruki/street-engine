use nodetype::TransportNodeType;
use numeric::Stage;

use crate::core::{
    container::path_network::NodeTrait,
    geometry::{angle::Angle, site::Site},
};

pub mod metrics;
pub mod nodetype;
pub mod numeric;

/// A node in the transport network.
#[derive(Debug, Clone, PartialEq)]
pub struct TransportNode {
    /// The site of the node.
    site: Site,

    /// The direction of the node.
    direction: Angle,

    /// The stage of the node.
    stage: Stage,

    /// The type of the node (e.g. land, bridge, tunnel, etc.)
    nodetype: TransportNodeType,
}

impl TransportNode {
    pub fn new(site: Site, direction: Angle, stage: Stage, nodetype: TransportNodeType) -> Self {
        Self {
            site,
            direction,
            stage,
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

impl NodeTrait for TransportNode {
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

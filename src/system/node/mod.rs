use metrics::TransportMetrics;
use nodetype::TransportNodeType;
use numeric::Stage;

use crate::core::geometry::site::Site;

pub mod metrics;
pub mod nodetype;
pub mod numeric;

/// A node in the transport network.
#[derive(Debug, Clone, PartialEq)]
pub struct TransportNode {
    site: Site,
    stage: Stage,
    metrics: TransportMetrics,
    nodetype: TransportNodeType,
}

impl TransportNode {
    pub fn new(
        site: Site,
        stage: Stage,
        metrics: TransportMetrics,
        nodetype: TransportNodeType,
    ) -> Self {
        Self {
            site,
            stage,
            metrics,
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

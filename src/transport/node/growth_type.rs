use crate::core::container::path_network::NodeId;

use super::transport_node::TransportNode;

#[derive(Debug)]
pub struct GrowthTypes {
    pub next_node: NextNodeType,
}

#[derive(Debug)]
pub enum NextNodeType {
    New(TransportNode),
    Existing(NodeId),
    Intersect(TransportNode, (NodeId, NodeId)),
    None,
}

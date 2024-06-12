use crate::core::container::path_network::NodeId;

use super::transport_node::TransportNode;

#[derive(Debug)]
pub enum GrowthType {
    New(TransportNode),
    Existing(NodeId),
    Intersect(TransportNode, (NodeId, NodeId)),
    None,
}

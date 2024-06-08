use crate::core::container::path_network::NodeId;

use super::transport_node::TransportNode;

#[derive(Debug)]
pub struct GrowthTypes {
    pub next_node: NextNodeType,
    pub bridge_node: BridgeNodeType,
}

#[derive(Debug)]
pub enum NextNodeType {
    New(TransportNode),
    Existing(NodeId),
    Intersect(TransportNode, (NodeId, NodeId)),
    None,
}

#[derive(Debug)]
pub enum BridgeNodeType {
    Middle(TransportNode),
    None,
}

impl BridgeNodeType {
    pub fn get_middle(&self) -> Option<&TransportNode> {
        match self {
            BridgeNodeType::Middle(node) => Some(node),
            BridgeNodeType::None => None,
        }
    }

    pub fn has_middle(&self) -> bool {
        match self {
            BridgeNodeType::Middle(_) => true,
            BridgeNodeType::None => false,
        }
    }

    pub fn is_none(&self) -> bool {
        match self {
            BridgeNodeType::Middle(_) => false,
            BridgeNodeType::None => true,
        }
    }
}

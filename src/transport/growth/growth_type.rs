use crate::{core::container::path_network::NodeId, transport::node::TransportNode};

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
    #[cfg(test)]
    pub fn is_none(&self) -> bool {
        match self {
            BridgeNodeType::Middle(_) => false,
            BridgeNodeType::None => true,
        }
    }
}

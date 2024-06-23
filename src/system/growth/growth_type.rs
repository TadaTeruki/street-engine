use crate::{core::container::path_network::NodeId, system::node::TransportNode};

/// The type of growth of a path in the transport network.
#[derive(Debug)]
pub enum GrowthType {
    /// Create a new path to the new node.
    New { new_node: TransportNode },

    /// Extend a new path to the existing node.
    Existing { existing_node_id: NodeId },

    /// Create a new intersection to the existing path.
    Intersection {
        new_node: TransportNode,
        path: (NodeId, NodeId),
    },

    /// Do not grow the path.
    None,
}

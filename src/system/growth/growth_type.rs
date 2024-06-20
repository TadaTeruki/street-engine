use crate::{core::container::path_network::NodeId, system::node::TransportNode};

/// The type of growth of a path in the transport network.
#[derive(Debug)]
pub enum GrowthType {
    /// Create a new path to the new node.
    New(TransportNode),

    /// Extend a new path to the existing node.
    Existing(NodeId),

    /// Create a new junction to the existing path.
    Junction(TransportNode, (NodeId, NodeId)),

    /// Do not grow the path.
    None,
}

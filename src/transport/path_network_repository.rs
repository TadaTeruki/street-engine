use std::collections::BTreeMap;

use crate::core::{
    container::path_network::{NodeId, PathNetwork},
    geometry::line_segment::LineSegment,
};

use super::node::transport_node::TransportNode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PathNetworkId(usize);

impl PathNetworkId {
    pub fn new(id: usize) -> Self {
        PathNetworkId(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PathNetworkGroup(usize);

impl PathNetworkGroup {
    pub fn new(group: usize) -> Self {
        PathNetworkGroup(group)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RelatedNode<'a> {
    pub node: &'a TransportNode,
    pub node_id: NodeId,
    pub network_id: PathNetworkId,
    pub group: PathNetworkGroup,
}

pub struct PathNetworkRepository {
    networks: BTreeMap<PathNetworkId, (PathNetwork<TransportNode>, PathNetworkGroup)>,
    last_network_id: PathNetworkId,
}

impl PathNetworkRepository {
    pub fn new() -> Self {
        PathNetworkRepository {
            networks: BTreeMap::new(),
            last_network_id: PathNetworkId(0),
        }
    }

    pub fn create_network(&mut self, group: PathNetworkGroup) -> PathNetworkId {
        let network_id = self.last_network_id;
        self.networks
            .insert(network_id, (PathNetwork::new(), group));
        self.last_network_id = PathNetworkId(network_id.0 + 1);
        network_id
    }

    pub fn get_network(&self, network_id: PathNetworkId) -> Option<&PathNetwork<TransportNode>> {
        self.networks.get(&network_id).map(|(network, _)| network)
    }

    pub fn modify_network<T>(
        &mut self,
        network_id: PathNetworkId,
        f: impl FnOnce(&mut PathNetwork<TransportNode>) -> T,
    ) -> Option<T> {
        if let Some(network) = self.networks.get_mut(&network_id) {
            Some(f(&mut network.0))
        } else {
            None
        }
    }

    // Find nodes around the line from the start site to the expected end site.
    pub fn related_nodes_iter(
        &self,
        line_segment: LineSegment,
        radius: f64,
    ) -> impl Iterator<Item = RelatedNode> {
        self.networks
            .iter()
            .flat_map(move |(network_id, (network, group))| {
                network
                    .nodes_around_line_iter(line_segment.clone(), radius)
                    .filter_map(move |node_id| -> Option<RelatedNode> {
                        Some(RelatedNode {
                            node: network.get_node(*node_id)?,
                            node_id: *node_id,
                            network_id: *network_id,
                            group: *group,
                        })
                    })
            })
    }

    // Find paths touching the rectangle around the line.
    pub fn related_paths_iter(
        &self,
        line_segment: LineSegment,
    ) -> impl Iterator<Item = (RelatedNode, RelatedNode)> {
        self.networks
            .iter()
            .flat_map(move |(network_id, (network, group))| {
                network
                    .paths_touching_rect_iter(line_segment.0, line_segment.1)
                    .filter_map(
                        move |(node_id_start, node_id_end)| -> Option<(RelatedNode, RelatedNode)> {
                            Some((
                                RelatedNode {
                                    node: network.get_node(*node_id_start)?,
                                    node_id: *node_id_start,
                                    network_id: *network_id,
                                    group: *group,
                                },
                                RelatedNode {
                                    node: network.get_node(*node_id_end)?,
                                    node_id: *node_id_end,
                                    network_id: *network_id,
                                    group: *group,
                                },
                            ))
                        },
                    )
            })
    }
}

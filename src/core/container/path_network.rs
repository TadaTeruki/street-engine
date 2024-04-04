use std::collections::BTreeMap;

use rstar::{PointDistance, RTree, RTreeObject};

use crate::core::geometry::{line_segment::LineSegment, site::Site};

use super::undirected::UndirectedGraph;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct NodeId(usize);

#[derive(Debug, Clone)]
struct PathTreeObject {
    line_segment: LineSegment,
    node_ids: (NodeId, NodeId),
}

impl RTreeObject for PathTreeObject {
    type Envelope = rstar::AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        rstar::AABB::from_corners(
            [self.line_segment.0.x, self.line_segment.0.y],
            [self.line_segment.1.x, self.line_segment.1.y],
        )
    }
}

impl PointDistance for PathTreeObject {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let site = Site::new(point[0], point[1]);
        let proj = self.line_segment.get_projection(&site);
        if let Some(proj) = proj {
            let dx = proj.x - site.x;
            let dy = proj.y - site.y;
            dx * dx + dy * dy
        } else {
            let start = &self.line_segment.0;
            let end = &self.line_segment.1;

            let d0 = start.distance(&Site::new(point[0], point[1]));
            let d1 = end.distance(&Site::new(point[0], point[1]));
            d0.min(d1)
        }
    }
}

impl PartialEq for PathTreeObject {
    fn eq(&self, other: &Self) -> bool {
        self.node_ids == other.node_ids || self.node_ids == (other.node_ids.1, other.node_ids.0)
    }
}

#[derive(Debug, Clone)]
struct PathNetwork<N>
where
    N: Eq + Copy + Into<Site>,
{
    nodes: BTreeMap<NodeId, N>,
    path_tree: RTree<PathTreeObject>,
    path_connection: UndirectedGraph<NodeId>,
    last_node_id: NodeId,
}

impl<N> PathNetwork<N>
where
    N: Eq + Copy + Into<Site>,
{
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            path_tree: RTree::new(),
            path_connection: UndirectedGraph::new(),
            last_node_id: NodeId(0),
        }
    }

    pub fn add_node(&mut self, node: N) -> NodeId {
        let node_id = self.last_node_id;
        self.nodes.insert(node_id, node);
        self.last_node_id = NodeId(node_id.0 + 1);
        node_id
    }

    pub fn remove_node(&mut self, node_id: NodeId) -> Option<NodeId> {
        let neighbors = if let Some(neighbors) = self.path_connection.neighbors_iter(node_id) {
            neighbors.copied().collect::<Vec<_>>()
        } else {
            return None;
        };

        neighbors.iter().for_each(|neighbor| {
            self.remove_path(node_id, *neighbor);
            if self.path_connection.neighbors_iter(*neighbor).is_none() {
                self.nodes.remove(neighbor);
            }
        });

        self.nodes.remove(&node_id);

        Some(node_id)
    }

    pub fn add_path(&mut self, from: NodeId, to: NodeId) -> Option<(NodeId, NodeId)> {
        if from == to {
            return None;
        }
        if self.path_connection.has_edge(from, to) {
            return None;
        }

        let (from_site, to_site) = if let (Some(from_node), Some(to_node)) =
            (self.nodes.get(&from), self.nodes.get(&to))
        {
            (*from_node, *to_node)
        } else {
            return None;
        };

        self.path_connection.add_edge(from, to);

        self.path_tree.insert(PathTreeObject {
            line_segment: LineSegment::new(from_site.into(), to_site.into()),
            node_ids: (from, to),
        });

        Some((from, to))
    }

    pub fn remove_path(&mut self, from: NodeId, to: NodeId) -> Option<(NodeId, NodeId)> {
        let (from_site, to_site) = if let (Some(from_node), Some(to_node)) =
            (self.nodes.get(&from), self.nodes.get(&to))
        {
            (*from_node, *to_node)
        } else {
            return None;
        };

        self.path_connection.remove_edge(from, to);

        self.path_tree.remove(&PathTreeObject {
            line_segment: LineSegment::new(from_site.into(), to_site.into()),
            node_ids: (from, to),
        });

        Some((from, to))
    }

    pub fn has_path(&self, from: NodeId, to: NodeId) -> bool {
        self.path_connection.has_edge(from, to)
    }

    fn check_path_state_is_consistent(&self) -> bool {
        self.path_tree.size() == self.path_connection.size()
            && self.path_connection.order() == self.nodes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_network() {
        let mut network = PathNetwork::new();
        let node0 = network.add_node(Site::new(0.0, 0.0));
        let node1 = network.add_node(Site::new(1.0, 1.0));
        let node2 = network.add_node(Site::new(2.0, 2.0));
        let node3 = network.add_node(Site::new(3.0, 3.0));
        let node4 = network.add_node(Site::new(1.0, 4.0));

        network.add_path(node0, node1);
        network.add_path(node1, node2);
        network.add_path(node2, node3);
        network.add_path(node3, node4);
        network.add_path(node4, node2);

        assert!(network.has_path(node0, node1));
        assert!(network.has_path(node1, node2));
        assert!(network.has_path(node2, node3));
        assert!(network.has_path(node3, node4));
        assert!(!network.has_path(node0, node2));

        assert!(network.check_path_state_is_consistent());

        network.remove_path(node1, node2);
        assert!(!network.has_path(node1, node2));
        assert!(network.has_path(node2, node3));

        assert!(network.check_path_state_is_consistent());

        network.remove_node(node1);
        assert!(!network.has_path(node0, node1));

        assert!(network.check_path_state_is_consistent());
    }
}

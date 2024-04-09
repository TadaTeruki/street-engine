use std::collections::BTreeMap;

use rstar::{PointDistance, RTree, RTreeObject};

use crate::core::geometry::{line_segment::LineSegment, site::Site};

use super::undirected::UndirectedGraph;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId(usize);

impl NodeId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

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
struct NodeTreeObject {
    site: Site,
    node_id: NodeId,
}

impl RTreeObject for NodeTreeObject {
    type Envelope = rstar::AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        rstar::AABB::from_point([self.site.x, self.site.y])
    }
}

impl PointDistance for NodeTreeObject {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let dx = self.site.x - point[0];
        let dy = self.site.y - point[1];
        dx * dx + dy * dy
    }
}

impl PartialEq for NodeTreeObject {
    fn eq(&self, other: &Self) -> bool {
        self.node_id == other.node_id
    }
}

#[derive(Debug, Clone)]
pub struct PathNetwork<N>
where
    N: Eq + Copy + Into<Site>,
{
    nodes: BTreeMap<NodeId, N>,
    path_tree: RTree<PathTreeObject>,
    node_tree: RTree<NodeTreeObject>,
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
            node_tree: RTree::new(),
            path_connection: UndirectedGraph::new(),
            last_node_id: NodeId(0),
        }
    }

    pub fn nodes_iter(&self) -> impl Iterator<Item = (NodeId, &N)> {
        self.nodes.iter().map(|(node_id, node)| (*node_id, node))
    }

    pub fn neighbors_iter(&self, node_id: NodeId) -> Option<impl Iterator<Item = (NodeId, &N)>> {
        self.path_connection
            .neighbors_iter(node_id)
            .map(|neighbors| {
                neighbors.filter_map(move |neighbor| Some((*neighbor, self.nodes.get(&neighbor)?)))
            })
    }

    pub(crate) fn add_node(&mut self, node: N) -> NodeId {
        let node_id = self.last_node_id;
        self.nodes.insert(node_id, node);
        self.node_tree.insert(NodeTreeObject {
            site: node.into(),
            node_id,
        });
        self.last_node_id = NodeId(node_id.0 + 1);
        node_id
    }

    pub(crate) fn remove_node(&mut self, node_id: NodeId) -> Option<NodeId> {
        let neighbors = if let Some(neighbors) = self.path_connection.neighbors_iter(node_id) {
            neighbors.copied().collect::<Vec<_>>()
        } else {
            return None;
        };

        let site = if let Some(node) = self.nodes.get(&node_id) {
            (*node).into()
        } else {
            return None;
        };

        neighbors.iter().for_each(|neighbor| {
            self.remove_path(node_id, *neighbor);
        });

        self.node_tree.remove(&NodeTreeObject { site, node_id });

        self.nodes.remove(&node_id);
        Some(node_id)
    }

    pub(crate) fn add_path(&mut self, from: NodeId, to: NodeId) -> Option<(NodeId, NodeId)> {
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

        let (from_site, to_site) = (from_site.into(), to_site.into());

        self.path_tree.insert(PathTreeObject {
            line_segment: LineSegment::new(from_site, to_site),
            node_ids: (from, to),
        });

        Some((from, to))
    }

    pub(crate) fn remove_path(&mut self, from: NodeId, to: NodeId) -> Option<(NodeId, NodeId)> {
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

    pub fn get_node(&self, node_id: NodeId) -> Option<&N> {
        self.nodes.get(&node_id)
    }

    pub fn has_path(&self, from: NodeId, to: NodeId) -> bool {
        self.path_connection.has_edge(from, to)
    }

    /// Search nodes around a site within a radius.
    pub fn nodes_around_site_iter(&self, site: Site, radius: f64) -> impl Iterator<Item = &NodeId> {
        self.nodes.iter().filter_map(move |(node_id, &node)| {
            if site.distance(&node.into()) <= radius {
                Some(node_id)
            } else {
                None
            }
        })
    }

    /// Search nodes around a line segment within a radius.
    pub fn nodes_around_line_iter(
        &self,
        line: LineSegment,
        radius: f64,
    ) -> impl Iterator<Item = &NodeId> {
        let envelope = rstar::AABB::from_corners(
            [
                line.0.x.min(line.1.x) - radius,
                line.0.y.min(line.1.y) - radius,
            ],
            [
                line.0.x.max(line.1.x) + radius,
                line.0.y.max(line.1.y) + radius,
            ],
        );
        self.node_tree
            .locate_in_envelope(&envelope)
            .filter(move |object| line.get_distance(&object.site) <= radius)
            .map(|object| &object.node_id)
    }

    /// Search paths touching a rectangle.
    pub fn paths_touching_rect_iter(
        &self,
        corner_0: Site,
        corner_1: Site,
    ) -> impl Iterator<Item = &(NodeId, NodeId)> {
        let search_rect =
            rstar::AABB::from_corners([corner_0.x, corner_0.y], [corner_1.x, corner_1.y]);

        self.path_tree
            .locate_in_envelope_intersecting(&search_rect)
            .map(|object| &object.node_ids)
    }

    pub fn into_optimized(self) -> Self {
        // TODO: optimize the path network
        self
    }

    /// Search paths crossing a line segment.
    /// Return the crossing paths and the intersection sites.
    fn paths_crossing_iter(&self, line: LineSegment) -> impl Iterator<Item = (&LineSegment, Site)> {
        let envelope = &PathTreeObject {
            line_segment: line.clone(),
            node_ids: (NodeId(0), NodeId(0)),
        }
        .envelope();
        self.path_tree
            .locate_in_envelope_intersecting(envelope)
            .filter_map(move |object| {
                object
                    .line_segment
                    .get_intersection(&line)
                    .map(|intersection| (&object.line_segment, intersection))
            })
    }

    /// Search paths around a site within a radius.
    fn paths_around_site_iter(
        &self,
        site: Site,
        radius: f64,
    ) -> impl Iterator<Item = &(NodeId, NodeId)> {
        self.path_tree
            .locate_within_distance([site.x, site.y], radius)
            .map(|object| &object.node_ids)
    }

    /// This function is only for testing
    #[allow(dead_code)]
    fn check_path_state_is_consistent(&self) -> bool {
        self.path_tree.size() == self.path_connection.size()
            && self.nodes.len() == self.node_tree.size()
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

    #[test]
    fn test_path_crossing_no_crosses() {
        let mut network = PathNetwork::new();
        let node0 = network.add_node(Site::new(0.0, 1.0));
        let node1 = network.add_node(Site::new(2.0, 3.0));
        let node2 = network.add_node(Site::new(4.0, 5.0));

        network.add_path(node0, node1);
        network.add_path(node1, node2);

        let path = LineSegment::new(Site::new(0.0, 0.95), Site::new(1.0, 1.95));
        let paths = network.paths_crossing_iter(path).collect::<Vec<_>>();
        assert_eq!(paths.len(), 0);

        let paths = network
            .paths_touching_rect_iter(Site::new(0.0, 0.0), Site::new(1.0, 1.0))
            .collect::<Vec<_>>();
        assert_eq!(paths.len(), 1);

        assert!(network.check_path_state_is_consistent());
    }

    #[test]
    fn test_path_crossing_all_cross() {
        let mut network = PathNetwork::new();

        let sites = vec![
            Site::new(0.0, 2.0),
            Site::new(2.0, 2.0),
            Site::new(2.0, 0.0),
            Site::new(0.0, 0.0),
        ];

        let nodes = sites
            .iter()
            .map(|site| network.add_node(*site))
            .collect::<Vec<_>>();

        for i in 0..sites.len() {
            // Add all paths between sites
            // When i == j, the path is expected to be ignored
            for j in i..sites.len() {
                network.add_path(nodes[i], nodes[j]);
            }
        }

        for i in 0..sites.len() {
            for j in 0..sites.len() {
                if i != j {
                    assert!(network.has_path(NodeId(i), NodeId(j)));
                }
            }
        }

        let path = LineSegment::new(Site::new(1.0, 3.0), Site::new(0.0, -1.0));
        let paths = network.paths_crossing_iter(path).collect::<Vec<_>>();
        assert_eq!(paths.len(), 4);

        let paths = network
            .paths_touching_rect_iter(Site::new(0.0, 0.0), Site::new(1.0, 2.0))
            .collect::<Vec<_>>();
        assert_eq!(paths.len(), 5);

        assert!(network.check_path_state_is_consistent());
    }

    #[test]
    fn test_path_crossing_at_endpoints() {
        let mut network = PathNetwork::new();
        let node0 = network.add_node(Site::new(0.0, 0.0));
        let node1 = network.add_node(Site::new(1.0, 1.0));
        let node2 = network.add_node(Site::new(1.0, -1.0));

        network.add_path(node0, node1);
        network.add_path(node1, node2);

        let path = LineSegment::new(Site::new(1.0, 1.0), Site::new(2.0, 2.0));
        let paths = network.paths_crossing_iter(path).collect::<Vec<_>>();
        assert_eq!(paths.len(), 1);

        assert!(network.check_path_state_is_consistent());
    }

    #[test]
    fn test_nodes_around_site() {
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

        let site = Site::new(1.0, 1.0);
        let nodes = network
            .nodes_around_site_iter(site, 1.0)
            .collect::<Vec<_>>();
        assert_eq!(nodes.len(), 1);

        let site = Site::new(2.0, 1.0);
        let nodes = network
            .nodes_around_site_iter(site, 2.0)
            .collect::<Vec<_>>();
        assert_eq!(nodes.len(), 2);

        let site = Site::new(2.0, 3.0);
        let nodes = network
            .nodes_around_site_iter(site, 2.0)
            .collect::<Vec<_>>();
        assert_eq!(nodes.len(), 3);

        let line = LineSegment::new(Site::new(1.0, 3.0), Site::new(3.0, 2.0));
        let nodes = network
            .nodes_around_line_iter(line, 1.0)
            .collect::<Vec<_>>();
        assert_eq!(nodes.len(), 3);

        let line = LineSegment::new(Site::new(1.0, 0.0), Site::new(0.0, 1.0));
        let nodes = network
            .nodes_around_line_iter(line, 2.5)
            .collect::<Vec<_>>();
        assert_eq!(nodes.len(), 3);

        network.remove_path(node3, node4);
        network.remove_node(node1);

        let site = Site::new(2.0, 1.0);
        let nodes = network
            .nodes_around_site_iter(site, 2.0)
            .collect::<Vec<_>>();
        assert_eq!(nodes.len(), 1);

        let line = LineSegment::new(Site::new(1.0, 0.0), Site::new(0.0, 1.0));
        let nodes = network
            .nodes_around_line_iter(line, 2.5)
            .collect::<Vec<_>>();
        assert_eq!(nodes.len(), 2);

        assert!(network.check_path_state_is_consistent());
    }

    #[test]
    fn test_complex_network() {
        let xorshift = |x: usize| -> usize {
            let mut x = x;
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            x
        };

        let sites = (0..100)
            .map(|i| Site::new(xorshift(i * 2) as f64, xorshift(i * 2 + 1) as f64))
            .collect::<Vec<_>>();

        let loop_count = 10;

        let mut network = PathNetwork::new();

        let nodeids = sites
            .iter()
            .map(|site| network.add_node(*site))
            .collect::<Vec<_>>();

        for l in 0..loop_count {
            let seed_start = l * sites.len() * sites.len();
            (0..sites.len()).for_each(|i| {
                (0..sites.len()).for_each(|j| {
                    let id = i * sites.len() + j;
                    if xorshift(id + seed_start) % 2 == 0 {
                        network.add_path(nodeids[i], nodeids[j]);
                    }
                });
            });

            assert!(network.check_path_state_is_consistent());

            (0..sites.len()).for_each(|i| {
                (0..sites.len()).for_each(|j| {
                    let id = i * sites.len() + j;
                    if xorshift(id + seed_start) % 3 == 0 {
                        network.remove_path(nodeids[i], nodeids[j]);
                    }
                });
            });

            assert!(network.check_path_state_is_consistent());
        }
    }
}

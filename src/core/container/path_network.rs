use std::collections::BTreeMap;

use rstar::RTree;

use crate::core::geometry::{line_segment::LineSegment, site::Site};

use super::{
    index_object::{NodeTreeObject, PathTreeObject},
    undirected::UndirectedGraph,
};

pub trait PathNetworkNodeTrait: Into<Site> + Copy + Eq {}
impl<T> PathNetworkNodeTrait for T where T: Into<Site> + Copy + Eq {}

/// ID for identifying a node in the network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId(usize);

impl NodeId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    fn as_num(&self) -> usize {
        self.0
    }
}

/// NodeIdGenerator is a simple struct that generates random ids.
///
/// This struct doesn't provide any methods to check the uniqueness of the generated ids.
#[derive(Debug, Clone, Default)]
pub struct NodeIdGenerator {
    next_id: usize,
}

impl NodeIdGenerator {
    pub fn generate_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        NodeId::new(id)
    }
}

/// Path network.
/// This struct is used to manage nodes and paths between nodes in 2D space.
///
/// This struct provides:
///  - functions to add, remove, and search nodes and paths.
///  - functions to search nodes around a site or a line segment.
#[derive(Debug, Clone)]
pub struct PathNetwork<N>
where
    N: PathNetworkNodeTrait,
{
    nodes: BTreeMap<NodeId, N>,
    path_tree: RTree<PathTreeObject<NodeId>>,
    node_tree: RTree<NodeTreeObject<NodeId>>,
    path_connection: UndirectedGraph<NodeId>,
    id_generator: NodeIdGenerator,
}

impl<N> Default for PathNetwork<N>
where
    N: PathNetworkNodeTrait,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<N> PathNetwork<N>
where
    N: PathNetworkNodeTrait,
{
    /// Create a new path network.
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            path_tree: RTree::new(),
            node_tree: RTree::new(),
            path_connection: UndirectedGraph::new(),
            id_generator: NodeIdGenerator::default(),
        }
    }
    /// Get nodes in the network.
    pub fn nodes_iter(&self) -> impl Iterator<Item = (NodeId, &N)> {
        self.nodes.iter().map(|(node_id, node)| (*node_id, node))
    }

    /// Get neighbors of a node.
    pub fn neighbors_iter(&self, node_id: NodeId) -> Option<impl Iterator<Item = (NodeId, &N)>> {
        self.path_connection
            .neighbors_iter(node_id)
            .map(|neighbors| {
                neighbors.filter_map(move |neighbor| Some((*neighbor, self.nodes.get(neighbor)?)))
            })
    }

    fn generate_id_with_check(&mut self) -> NodeId {
        let mut node_id = self.id_generator.generate_id();
        while self.nodes.contains_key(&node_id) {
            node_id = self.id_generator.generate_id();
        }
        node_id
    }

    /// Add a node to the network.
    pub(crate) fn add_node(&mut self, node: N) -> NodeId {
        let node_id = self.generate_id_with_check();
        self.nodes.insert(node_id, node);
        self.node_tree
            .insert(NodeTreeObject::new(node.into(), node_id));
        node_id
    }

    /// Remove a node from the network.
    /// This function can be never used, but it is kept for future use.
    #[allow(dead_code)]
    fn remove_node(&mut self, node_id: NodeId) -> Option<NodeId> {
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

        self.node_tree.remove(&NodeTreeObject::new(site, node_id));

        self.nodes.remove(&node_id);
        Some(node_id)
    }

    /// Add a path to the network.
    pub(crate) fn add_path(&mut self, start: NodeId, end: NodeId) -> Option<(NodeId, NodeId)> {
        if start == end {
            return None;
        }
        if self.path_connection.has_edge(start, end) {
            return None;
        }

        let (start_site, end_site) = if let (Some(start_node), Some(end_node)) =
            (self.nodes.get(&start), self.nodes.get(&end))
        {
            (*start_node, *end_node)
        } else {
            return None;
        };

        self.path_connection.add_edge(start, end);

        let (start_site, end_site) = (start_site.into(), end_site.into());

        self.path_tree.insert(PathTreeObject::new(
            LineSegment::new(start_site, end_site),
            (start, end),
        ));

        Some((start, end))
    }

    /// Remove a path from the network.
    pub(crate) fn remove_path(&mut self, start: NodeId, end: NodeId) -> Option<(NodeId, NodeId)> {
        let (start_site, end_site) = if let (Some(start_node), Some(end_node)) =
            (self.nodes.get(&start), self.nodes.get(&end))
        {
            (*start_node, *end_node)
        } else {
            return None;
        };

        self.path_connection.remove_edge(start, end);

        self.path_tree.remove(&PathTreeObject::new(
            LineSegment::new(start_site.into(), end_site.into()),
            (start, end),
        ));

        Some((start, end))
    }

    /// Get a node by its NodeId.
    pub fn get_node(&self, node_id: NodeId) -> Option<&N> {
        self.nodes.get(&node_id)
    }

    /// Check if there is a path between two nodes.
    pub fn has_path(&self, start: NodeId, to: NodeId) -> bool {
        self.path_connection.has_edge(start, to)
    }

    /// Search nodes around a site within a radius.
    pub fn nodes_around_site_iter(&self, site: Site, radius: f64) -> impl Iterator<Item = &NodeId> {
        let envelope = rstar::AABB::from_corners(
            [site.x - radius, site.y - radius],
            [site.x + radius, site.y + radius],
        );
        self.node_tree
            .locate_in_envelope(&envelope)
            .filter(move |object| site.distance(object.site()) <= radius)
            .map(|object| object.node_id())
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
            .filter(move |object| line.get_distance(object.site()) <= radius)
            .map(|object| object.node_id())
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
            .map(|object| object.node_ids())
    }

    /// Parse the network into a list of nodes and paths.
    ///
    /// This function is not exposed now, but it may be useful in the future.
    fn parse(&self) -> (Vec<N>, Vec<(usize, usize)>) {
        let nodes = self.nodes.values().cloned().collect::<Vec<_>>();
        let paths = self
            .path_tree
            .iter()
            .map(|object| {
                let (start, end) = object.node_ids();
                (start.as_num(), end.as_num())
            })
            .collect::<Vec<_>>();
        (nodes, paths)
    }

    pub fn from(nodes: Vec<N>, paths: &[(usize, usize)]) -> Option<Self> {
        let mut id_generator = NodeIdGenerator::default();

        // distribute NodeIds to nodes
        let nodes = nodes
            .into_iter()
            .map(|node| (id_generator.generate_id(), node))
            .collect::<Vec<_>>();

        // original paths length
        let paths_len = paths.len();

        // convert paths from usize to NodeId
        let paths = paths
            .iter()
            .filter_map(|(start, end)| Some((nodes.get(*start)?.0, nodes.get(*end)?.0)))
            .collect::<Vec<_>>();
        // if there are invalid paths, return None
        if paths.len() != paths_len {
            return None;
        }

        let node_into_site = |node: N| -> Site { Into::<Site>::into(node) };

        // rtree for nodes
        let node_tree = RTree::bulk_load(
            nodes
                .iter()
                .map(|(node_id, node)| NodeTreeObject::new(node_into_site(*node), *node_id))
                .collect::<Vec<_>>(),
        );

        let path_connection = paths.iter().fold(
            UndirectedGraph::new(),
            |mut path_connection, (start, end)| {
                path_connection.add_edge(*start, *end);
                path_connection
            },
        );

        let nodes = nodes.into_iter().collect::<BTreeMap<_, _>>();

        let path_tree = RTree::bulk_load(
            paths
                .iter()
                .filter_map(|(start, end)| {
                    let (start_site, end_site) = (
                        node_into_site(*nodes.get(start)?),
                        node_into_site(*nodes.get(end)?),
                    );
                    Some(PathTreeObject::new(
                        LineSegment::new(start_site, end_site),
                        (*start, *end),
                    ))
                })
                .collect::<Vec<_>>(),
        );
        // if there are invalid paths, return None
        if path_tree.size() != paths_len {
            return None;
        }
        Some(Self {
            nodes,
            path_tree,
            node_tree,
            path_connection,
            id_generator,
        })
    }

    /// Get the optimized path network.
    pub fn reconstruct(self) -> Option<Self> {
        let (nodes, paths) = self.parse();
        Self::from(nodes, &paths)
    }

    /// This function is only for testing
    #[cfg(test)]
    fn check_path_state_is_consistent(&self) -> bool {
        self.path_tree.size() == self.path_connection.size()
            && self.nodes.len() == self.node_tree.size()
    }

    /// Search the nearest node from a site.
    /// This function is only for testing, but can be exposed in the future.
    #[cfg(test)]
    pub fn search_nearest_node(&self, site: Site) -> Option<NodeId> {
        self.node_tree
            .nearest_neighbor(&[site.x, site.y])
            .map(|object| *object.node_id())
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

        let paths = network
            .paths_touching_rect_iter(Site::new(0.0, 0.0), Site::new(1.0, 2.0))
            .collect::<Vec<_>>();
        assert_eq!(paths.len(), 5);

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

    fn xorshift(x: usize) -> usize {
        let mut x = x;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        x
    }

    #[test]
    fn test_reconstruction() {
        let nodes = (0..300)
            .map(|i| Site::new(i as f64, i as f64))
            .collect::<Vec<_>>();

        let paths = {
            let mut paths = Vec::new();
            for i in 0..nodes.len() {
                for j in i + 1..nodes.len() {
                    if xorshift(i * nodes.len() + j) % 2 == 0 {
                        paths.push((i, j));
                    }
                }
            }
            paths
        };

        let mut network0: PathNetwork<Site> = PathNetwork::new();
        let nodeids0 = nodes
            .clone()
            .into_iter()
            .map(|node| network0.add_node(node))
            .collect::<Vec<_>>();

        for (start, end) in paths.iter() {
            network0.add_path(nodeids0[*start], nodeids0[*end]);
        }

        let network1: PathNetwork<Site> = PathNetwork::from(nodes.clone(), &paths).unwrap();
        let nodeids1 = nodes
            .iter()
            .map(|node| network1.search_nearest_node(*node).unwrap())
            .collect::<Vec<_>>();

        let network2 = network0.clone().reconstruct().unwrap();
        let nodeids2 = nodes
            .iter()
            .map(|node| network2.search_nearest_node(*node).unwrap())
            .collect::<Vec<_>>();

        let network3 = network1.clone().reconstruct().unwrap();
        let nodeids3 = nodes
            .iter()
            .map(|node| network3.search_nearest_node(*node).unwrap())
            .collect::<Vec<_>>();

        for i in 0..nodes.len() {
            for j in 0..nodes.len() {
                let r0 = network0.has_path(nodeids0[i], nodeids0[j]);
                let r1 = network1.has_path(nodeids1[i], nodeids1[j]);
                assert_eq!(r0, r1);

                let r2 = network2.has_path(nodeids2[i], nodeids2[j]);
                assert_eq!(r1, r2);

                let r3 = network3.has_path(nodeids3[i], nodeids3[j]);
                assert_eq!(r2, r3);
            }
        }
    }
}
